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

impl From<IpAddr> for embedded_nal::IpAddr {
    fn from(s: IpAddr) -> embedded_nal::IpAddr {
        match s.0 {
            net::IpAddr::V4(i) => i.octets().into(),
            net::IpAddr::V6(i) => i.octets().into(),
        }
    }
}

impl From<IpAddr> for net::IpAddr {
    fn from(s: IpAddr) -> net::IpAddr {
        s.0
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

impl From<SocketAddr> for net::SocketAddr {
    fn from(s: SocketAddr) -> net::SocketAddr {
        s.0
    }
}

impl From<embedded_nal::SocketAddr> for SocketAddr {
    fn from(input: embedded_nal::SocketAddr) -> Self {
        Self((IpAddr::from(input.ip()).0, input.port()).into())
    }
}

impl From<SocketAddr> for embedded_nal::SocketAddr {
    fn from(s: SocketAddr) -> embedded_nal::SocketAddr {
        (IpAddr(s.0.ip()), s.0.port()).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equal_ipv6() {
        let nal: embedded_nal::IpAddr = "2001:db8::17".parse().unwrap();
        let native: std::net::IpAddr = "2001:db8::17".parse().unwrap();

        let converted_to_nal: embedded_nal::IpAddr = IpAddr(native).into();
        assert_eq!(nal, converted_to_nal);

        let converted_to_native: std::net::IpAddr = IpAddr::from(nal).into();
        assert_eq!(native, converted_to_native);
    }

    #[test]
    fn equal_ipv4() {
        let nal: embedded_nal::IpAddr = "192.0.2.42".parse().unwrap();
        let native: std::net::IpAddr = "192.0.2.42".parse().unwrap();

        let converted_to_nal: embedded_nal::IpAddr = IpAddr(native).into();
        assert_eq!(nal, converted_to_nal);

        let converted_to_native: std::net::IpAddr = IpAddr::from(nal).into();
        assert_eq!(native, converted_to_native);
    }

    #[test]
    fn equal_port() {
        let nal: embedded_nal::SocketAddr = "[2001:db8::17]:42".parse().unwrap();
        let native: std::net::SocketAddr = "[2001:db8::17]:42".parse().unwrap();

        let converted_to_nal: embedded_nal::SocketAddr = SocketAddr(native).into();
        assert_eq!(nal, converted_to_nal);

        let converted_to_native: std::net::SocketAddr = SocketAddr::from(nal).into();
        assert_eq!(native, converted_to_native);
    }
}
