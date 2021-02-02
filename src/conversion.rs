use embedded_nal::nb;
use std::io;
use std::net;
use std::net::ToSocketAddrs;
use std::option::IntoIter;

pub(crate) fn to_nb(e: io::Error) -> nb::Error<io::Error> {
    use io::ErrorKind::{TimedOut, WouldBlock};
    match e.kind() {
        WouldBlock | TimedOut => nb::Error::WouldBlock,
        _ => e.into(),
    }
}

/// Wrapper around the `std` IP address type that converts to `non_std`
/// counterpart and vice versa.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IpAddr(net::IpAddr);

impl From<embedded_nal::IpAddr> for IpAddr {
    fn from(input: embedded_nal::IpAddr) -> Self {
        match input {
            embedded_nal::IpAddr::V4(i) => Self(i.octets().into()),
            embedded_nal::IpAddr::V6(i) => Self(i.octets().into()),
        }
    }
}

impl Into<embedded_nal::IpAddr> for IpAddr {
    fn into(self) -> embedded_nal::IpAddr {
        match self.0 {
            net::IpAddr::V4(i) => i.octets().into(),
            net::IpAddr::V6(i) => i.octets().into(),
        }
    }
}

impl Into<net::IpAddr> for IpAddr {
    fn into(self) -> net::IpAddr {
        self.0
    }
}

/// Wrapper around the `std` socket address type that converts to `non_std`
/// counterpart and vice versa.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SocketAddr(net::SocketAddr);

impl ToSocketAddrs for SocketAddr {
    type Iter = IntoIter<net::SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<IntoIter<net::SocketAddr>> {
        self.0.to_socket_addrs()
    }
}

impl From<net::SocketAddr> for SocketAddr {
    fn from(input: net::SocketAddr) -> Self {
        Self(input)
    }
}

impl Into<net::SocketAddr> for SocketAddr {
    fn into(self) -> net::SocketAddr {
        self.0
    }
}

impl From<embedded_nal::SocketAddr> for SocketAddr {
    fn from(input: embedded_nal::SocketAddr) -> Self {
        Self((IpAddr::from(input.ip()).0, input.port()).into())
    }
}

impl Into<embedded_nal::SocketAddr> for SocketAddr {
    fn into(self) -> embedded_nal::SocketAddr {
        (IpAddr(self.0.ip()), self.0.port()).into()
    }
}
