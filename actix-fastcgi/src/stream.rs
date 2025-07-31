//! Socket Connection Abstraction with Support for Unix/TCP

use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpStream, UnixStream},
};

use super::error::Error;

/// Default socket address on failure to parse configured address
pub(crate) const DEFAULT_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000);

/// Compiled Unix/TCP Socket Address
#[derive(Clone)]
pub enum StreamAddr {
    Unix(PathBuf),
    Tcp(Vec<SocketAddr>),
}

impl From<SocketAddr> for StreamAddr {
    fn from(value: SocketAddr) -> Self {
        Self::Tcp(vec![value])
    }
}

impl FromStr for StreamAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (scheme, addr) = s.split_once("://").unwrap_or(("tcp", s));
        match &scheme.to_lowercase() == "unix" {
            true => Ok(Self::Unix(PathBuf::from(addr))),
            false => Ok(Self::Tcp(addr.to_socket_addrs()?.collect())),
        }
    }
}

/// Socket abstraction on [`TcpStream`](tokio::net::TcpStream) or
/// [`UnixStream`](tokio::net::UnixStream)
#[pin_project(project = AbsStreamProj)]
pub enum SockStream {
    Unix(#[pin] UnixStream),
    Tcp(#[pin] TcpStream),
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
    pub async fn connect(addr: &StreamAddr) -> Result<Self, Error> {
        match addr {
            StreamAddr::Unix(addr) => Ok(Self::Unix(UnixStream::connect(addr).await?)),
            StreamAddr::Tcp(addr) => Ok(Self::Tcp(TcpStream::connect(&addr[..]).await?)),
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
            AbsStreamProj::Tcp(t) => t.poll_read(cx, buf),
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
            AbsStreamProj::Tcp(t) => t.poll_write(cx, buf),
        }
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        match self.project() {
            AbsStreamProj::Unix(u) => u.poll_flush(cx),
            AbsStreamProj::Tcp(t) => t.poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            AbsStreamProj::Unix(u) => u.poll_shutdown(cx),
            AbsStreamProj::Tcp(t) => t.poll_shutdown(cx),
        }
    }
}
