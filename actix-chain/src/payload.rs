use std::{
    cell::{RefCell, RefMut},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_web::{
    dev::Payload,
    error::PayloadError,
    web::{Bytes, BytesMut},
};
use futures_core::{Stream, stream::LocalBoxStream};

pub(crate) struct PayloadRef(Rc<RefCell<PayloadBuffer>>);

impl PayloadRef {
    pub fn new<S>(stream: S, buffer_size: usize) -> Self
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        Self::from(PayloadBuffer {
            stream: Box::pin(stream),
            buf: BytesMut::with_capacity(1_024), // pre-allocate 1KiB
            eof: false,
            overflow: false,
            cursor: 0,
            body_buffer_size: buffer_size,
        })
    }

    #[inline]
    pub fn get_mut(&self) -> RefMut<'_, PayloadBuffer> {
        self.0.borrow_mut()
    }

    #[inline]
    pub fn into_stream(&self) -> LocalBoxStream<'static, Result<Bytes, PayloadError>> {
        Box::pin(self.clone())
    }

    pub fn into_payload(&self) -> Payload {
        Payload::Stream {
            payload: self.into_stream(),
        }
    }
}

impl From<PayloadBuffer> for PayloadRef {
    fn from(value: PayloadBuffer) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl Clone for PayloadRef {
    fn clone(&self) -> PayloadRef {
        Self(Rc::clone(&self.0))
    }
}

//TODO: implement max-body-size which writes to file buffer
// to support beyond memory-limit

/// Payload buffer.
pub(crate) struct PayloadBuffer {
    pub(crate) stream: LocalBoxStream<'static, Result<Bytes, PayloadError>>,
    pub(crate) buf: BytesMut,

    pub(crate) eof: bool,
    pub(crate) overflow: bool,

    pub(crate) cursor: usize,
    pub(crate) body_buffer_size: usize,
}

impl PayloadBuffer {
    #[inline]
    pub(crate) fn reset_stream(&mut self) {
        self.cursor = 0;
    }

    fn read_buffered(&mut self) -> Option<Bytes> {
        if self.cursor < self.buf.len() {
            let data = self
                .buf
                .clone()
                .split_to(self.buf.len() - self.cursor)
                .freeze();
            self.cursor += data.len();
            return Some(data);
        }
        None
    }
}

impl Stream for PayloadRef {
    type Item = Result<Bytes, PayloadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.get_mut().get_mut();
        if let Some(data) = this.read_buffered() {
            return Poll::Ready(Some(Ok(data)));
        }
        if this.eof {
            return Poll::Ready(None);
        }
        if this.overflow {
            return Poll::Ready(Some(Err(PayloadError::Overflow)));
        }
        match Pin::new(&mut this.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => {
                if this.cursor + data.len() > this.body_buffer_size {
                    this.overflow = true;
                    return Poll::Ready(Some(Err(PayloadError::Overflow)));
                }
                this.buf.extend_from_slice(&data);
                this.cursor += data.len();
                Poll::Ready(Some(Ok(data)))
            }
            Poll::Ready(None) => {
                this.eof = true;
                Poll::Ready(None)
            }
            status => status,
        }
    }
}
