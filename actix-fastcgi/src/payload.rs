//! Stream Abstraction for FastCGI

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    HttpMessage, HttpResponse,
    dev::ServiceRequest,
    error::PayloadError,
    http::StatusCode,
    web::{Bytes, BytesMut},
};
use fastcgi_client::{ClientError, response::Content};
use futures_core::{Stream, stream::LocalBoxStream};
use futures_util::StreamExt;
use tokio_util::io::StreamReader;

use super::error::Error;

const STATUS_HEADER: &str = "Status";

/// Request Stream wrapper for converting
/// [`ServiceRequest`](actix_web::dev::ServiceRequest) into
/// [`StreamReader`](tokio_util::io::StreamReader)
pub struct RequestStream(LocalBoxStream<'static, Result<Bytes, PayloadError>>);

impl RequestStream {
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        Self(Box::pin(stream))
    }
    #[inline]
    pub fn from_request(req: &mut ServiceRequest) -> Self {
        Self::new(req.take_payload())
    }
    #[inline]
    pub fn into_reader(self) -> StreamReader<Self, Bytes> {
        StreamReader::new(self)
    }
}

impl Stream for RequestStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => {
                tracing::trace!("data! {data:?}");
                Poll::Ready(Some(Ok(data)))
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(io::Error::other(err)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Response Stream buffer for converting
/// [`StreamResponse`](fastcgi_client::response::ResponseStream) into
/// [`HttpResponse`](actix_web::HttpResponse)
pub struct ResponseStream {
    stream: LocalBoxStream<'static, Result<Content, ClientError>>,
    buf: BytesMut,
    eof: Option<usize>,
}

impl ResponseStream {
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Content, ClientError>> + 'static,
    {
        Self {
            stream: Box::pin(stream),
            buf: BytesMut::with_capacity(1_024), // pre-allocate 1KiB
            eof: None,
        }
    }

    #[inline]
    async fn read_until_body(&mut self) -> Result<(), Error> {
        while self.eof.is_none() {
            match self.next().await {
                Some(item) => item?,
                None => return Err(Error::UnexpectedEnd),
            };
        }
        Ok(())
    }

    /// Convert Stream Buffer into HttpResponse
    ///
    /// Internally writes stream stdout to temporary memory-buffer
    /// until all headers can be read, then passes the rest of the stream
    /// body directly until final EOF is reached.
    pub async fn into_response(mut self) -> Result<HttpResponse, Error> {
        self.read_until_body().await?;

        let raw_headers = self.buf.split_to(self.eof.expect("missing eof") + 4);
        let mut headers = [httparse::EMPTY_HEADER; 32];
        httparse::parse_headers(&raw_headers[..raw_headers.len() - 2], &mut headers)
            .map_err(Error::InvalidHeaders)?;

        let mut builder = HttpResponse::Ok();
        for header in headers.into_iter().filter(|h| !h.name.is_empty()) {
            match header.name {
                STATUS_HEADER => {
                    let mut split = header.value.split(|b| b.is_ascii_whitespace());
                    builder.status(StatusCode::from_bytes(split.next().unwrap_or(b""))?)
                }
                name => builder.append_header((name, header.value)),
            };
        }

        Ok(builder.streaming(self))
    }
}

impl Stream for ResponseStream {
    type Item = Result<Bytes, PayloadError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.buf.is_empty() {
            let idx = self.buf.len();
            return Poll::Ready(Some(Ok(self.buf.split_to(idx).freeze())));
        }
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(content))) => match content {
                Content::Stdout(data) => {
                    if self.eof.is_none() {
                        self.buf.extend_from_slice(&data);
                        self.eof = data.windows(4).position(|w| w == b"\r\n\r\n");
                    }
                    Poll::Ready(Some(Ok(data)))
                }
                Content::Stderr(data) => {
                    let message = std::str::from_utf8(&data);
                    tracing::warn!("FastCGI Stderr {message:?}");
                    Poll::Ready(Some(Ok(Bytes::new())))
                }
            },
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(PayloadError::Io(
                io::Error::other(err.to_string()),
            )))),
            Poll::Ready(None) => {
                self.eof = Some(self.buf.len());
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
