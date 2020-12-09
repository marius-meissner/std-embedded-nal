use embedded_nal::{AddrType, Dns, IpAddr};
use heapless::consts::U256;
use heapless::String;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, ToSocketAddrs};

/// An std::io::Error compatible error type constructable when to_socket_addrs comes up empty
/// (because it does not produce an error of its own)
#[derive(Debug)]
struct NotFound;

impl Display for NotFound {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Not found")
    }
}

impl error::Error for NotFound {}

impl Dns for crate::Stack {
    type Error = Error;

    fn gethostbyname(&self, hostname: &str, addr_type: AddrType) -> Result<IpAddr> {
        let with_fake_port = if hostname.find(':').is_some() {
            format!("[{}]:1234", hostname)
        } else {
            format!("{}:1234", hostname)
        };

        let accept_v4 = addr_type != AddrType::IPv6;
        let accept_v6 = addr_type != AddrType::IPv4;

        for addr in with_fake_port.to_socket_addrs()? {
            match addr {
                SocketAddr::V4(v) if accept_v4 => {
                    return Ok(v.ip().octets().into());
                }
                SocketAddr::V6(v) if accept_v6 => {
                    return Ok(v.ip().octets().into());
                }
                _ => continue,
            }
        }

        Err(Error::new(ErrorKind::NotFound, NotFound))
    }

    fn gethostbyaddr(&self, _addr: IpAddr) -> Result<String<U256>> {
        Err(Error::new(ErrorKind::NotFound, NotFound))
    }
}
