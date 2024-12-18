use embedded_nal::{nb, AddrType, Dns};
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

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

    fn get_host_by_name(
        &mut self,
        hostname: &str,
        addr_type: AddrType,
    ) -> Result<IpAddr, nb::Error<Error>> {
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

        Err(nb::Error::Other(Error::new(ErrorKind::NotFound, NotFound)))
    }

    fn get_host_by_address(
        &mut self,
        _addr: IpAddr,
        _result: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        nb::Result::Err(nb::Error::Other(Error::new(ErrorKind::NotFound, NotFound)))
    }
}
