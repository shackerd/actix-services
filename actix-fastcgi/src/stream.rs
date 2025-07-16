//! Socket Connection Abstraction with Support for Unix/TCP

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpStream, UnixStream},
};

use super::error::Error;

/// Socket abstraction on [`TcpStream`](tokio::net::TcpStream) or
/// [`UnixStream`](tokio::net::UnixStream)
#[pin_project(project = AbsStreamProj)]
pub enum SockStream {
    Unix(#[pin] UnixStream),
    TCP(#[pin] TcpStream),
}

impl SockStream {
    /// Connect to the relevant unix/tcp socket using a connection uri
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_fastcgi::{SockStream, Error};
    ///
    /// async fn connect() -> Result<(), Error> {
    ///   let unix = SockStream::connect("unix:///var/run/program.sock").await?;
    ///   let tcp  = SockStream::connect("tcp://localhost:9000").await?;
    ///   let tcp2 = SockStream::connect("192.168.0.2:9000").await?;
    ///   Ok(())
    /// }
    /// ```
    pub async fn connect(addr: &str) -> Result<Self, Error> {
        let (scheme, addr) = addr.split_once("://").unwrap_or(("tcp", addr));
        match &scheme.to_lowercase() == "unix" {
            true => Ok(Self::Unix(UnixStream::connect(addr).await?)),
            false => Ok(Self::TCP(TcpStream::connect(addr).await?)),
        }
    }
}

impl AsyncRead for SockStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.project() {
            AbsStreamProj::Unix(u) => u.poll_read(cx, buf),
            AbsStreamProj::TCP(t) => t.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SockStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        match self.project() {
            AbsStreamProj::Unix(u) => u.poll_write(cx, buf),
            AbsStreamProj::TCP(t) => t.poll_write(cx, buf),
        }
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        match self.project() {
            AbsStreamProj::Unix(u) => u.poll_flush(cx),
            AbsStreamProj::TCP(t) => t.poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            AbsStreamProj::Unix(u) => u.poll_shutdown(cx),
            AbsStreamProj::TCP(t) => t.poll_shutdown(cx),
        }
    }
}
