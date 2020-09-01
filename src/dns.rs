use std::net::ToSocketAddrs;

/// An std::io::Error compatible error type constructable when to_socket_addrs comes up empty
/// (because it does not produce an error of its own)
#[derive(Debug)]
struct NotFound;

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Not found")
    }
}

impl std::error::Error for NotFound {
}

impl embedded_nal::Dns for crate::Stack {
    type Error = std::io::Error;

    fn gethostbyname(&self, hostname: &str, addr_type: embedded_nal::AddrType) -> Result<embedded_nal::IpAddr, Self::Error> {
        let with_fake_port = if hostname.find(':').is_some() {
            format!("[{}]:1234", hostname)
        } else {
            format!("{}:1234", hostname)
        };

        let accept_v4 = addr_type != embedded_nal::AddrType::IPv6;
        let accept_v6 = addr_type != embedded_nal::AddrType::IPv4;

        for addr in with_fake_port.to_socket_addrs() {
            for deep in addr {
                match deep {
                    std::net::SocketAddr::V4(v) if accept_v4 => { return Ok(v.ip().octets().into()); }
                    std::net::SocketAddr::V6(v) if accept_v6 => { return Ok(v.ip().octets().into()); }
                    _ => continue
                }
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::NotFound, NotFound))
    }

    fn gethostbyaddr(&self, _addr: embedded_nal::IpAddr) -> Result<heapless::String<heapless::consts::U256>, Self::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, NotFound))
    }
}
