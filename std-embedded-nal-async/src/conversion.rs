use std::io;
use std::net;
use std::net::ToSocketAddrs;
use std::option::IntoIter;

/// Wrapper around the `std` IP address type that converts to `non_std`
/// counterpart and vice versa.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IpAddr(net::IpAddr);

impl From<embedded_nal_async::IpAddr> for IpAddr {
    fn from(input: embedded_nal_async::IpAddr) -> Self {
        match input {
            embedded_nal_async::IpAddr::V4(i) => Self(i.octets().into()),
            embedded_nal_async::IpAddr::V6(i) => Self(i.octets().into()),
        }
    }
}

// That this is missing zone info on IPv6 probably just makes std::net::IpAddr a bad intermediate
// type.
impl From<nix::libc::in6_pktinfo> for IpAddr {
    fn from(input: nix::libc::in6_pktinfo) -> Self {
        // FIXME why isn't this having zone infos??
        Self(input.ipi6_addr.s6_addr.into())
    }
}

impl From<IpAddr> for nix::libc::in6_pktinfo {
    fn from(input: IpAddr) -> nix::libc::in6_pktinfo {
        let input = match input.0 {
            std::net::IpAddr::V6(a) => a,
            _ => panic!("Type requires IPv6 addresses"),
        };
        nix::libc::in6_pktinfo {
            ipi6_addr: nix::libc::in6_addr {
                s6_addr: input.octets(),
            },
            // FIXME and here it really hurts
            ipi6_ifindex: 0,
        }
    }
}

impl From<nix::libc::in_pktinfo> for IpAddr {
    fn from(input: nix::libc::in_pktinfo) -> Self {
        // FIXME discarding interface index?
        Self(net::Ipv4Addr::from(input.ipi_spec_dst.s_addr).into())
    }
}

impl From<IpAddr> for nix::libc::in_pktinfo {
    fn from(input: IpAddr) -> nix::libc::in_pktinfo {
        let input = match input.0 {
            std::net::IpAddr::V4(a) => a,
            _ => panic!("Type requires IPv4 addresses"),
        };
        nix::libc::in_pktinfo {
            ipi_spec_dst: nix::libc::in_addr {
                s_addr: input.into(),
            },
            ipi_addr: nix::libc::in_addr {
                s_addr: input.into(),
            },
            // FIXME and here it really hurts
            ipi_ifindex: 0,
        }
    }
}

impl From<IpAddr> for embedded_nal_async::IpAddr {
    fn from(s: IpAddr) -> embedded_nal_async::IpAddr {
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

impl From<embedded_nal_async::SocketAddr> for SocketAddr {
    fn from(input: embedded_nal_async::SocketAddr) -> Self {
        Self((IpAddr::from(input.ip()).0, input.port()).into())
    }
}

impl From<SocketAddr> for embedded_nal_async::SocketAddr {
    fn from(s: SocketAddr) -> embedded_nal_async::SocketAddr {
        (IpAddr(s.0.ip()), s.0.port()).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equal_ipv6() {
        let nal: embedded_nal_async::IpAddr = "2001:db8::17".parse().unwrap();
        let native: std::net::IpAddr = "2001:db8::17".parse().unwrap();

        let converted_to_nal: embedded_nal_async::IpAddr = IpAddr(native).into();
        assert_eq!(nal, converted_to_nal);

        let converted_to_native: std::net::IpAddr = IpAddr::from(nal).into();
        assert_eq!(native, converted_to_native);
    }

    #[test]
    fn equal_ipv4() {
        let nal: embedded_nal_async::IpAddr = "192.0.2.42".parse().unwrap();
        let native: std::net::IpAddr = "192.0.2.42".parse().unwrap();

        let converted_to_nal: embedded_nal_async::IpAddr = IpAddr(native).into();
        assert_eq!(nal, converted_to_nal);

        let converted_to_native: std::net::IpAddr = IpAddr::from(nal).into();
        assert_eq!(native, converted_to_native);
    }

    #[test]
    fn equal_port() {
        let nal: embedded_nal_async::SocketAddr = "[2001:db8::17]:42".parse().unwrap();
        let native: std::net::SocketAddr = "[2001:db8::17]:42".parse().unwrap();

        let converted_to_nal: embedded_nal_async::SocketAddr = SocketAddr(native).into();
        assert_eq!(nal, converted_to_nal);

        let converted_to_native: std::net::SocketAddr = SocketAddr::from(nal).into();
        assert_eq!(native, converted_to_native);
    }
}
