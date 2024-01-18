//! DNS implementation based on async_std

use async_std::net::{SocketAddr, ToSocketAddrs};
use embedded_nal_async::{AddrType, IpAddr};

/// An std::io::Error compatible error type constructable when to_socket_addrs comes up empty
/// (because it does not produce an error of its own)
#[derive(Debug)]
struct NotFound;

impl core::fmt::Display for NotFound {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Not found")
    }
}

impl std::error::Error for NotFound {}

/// An std::io::Error compatible error type expressing that a name doesn't fit in the
/// provided response buffer.
#[derive(Debug)]
struct TooLong;

impl core::fmt::Display for TooLong {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Name too long")
    }
}

impl std::error::Error for TooLong {}

impl embedded_nal_async::Dns for crate::Stack {
    type Error = std::io::Error;

    async fn get_host_by_name(
        &self,
        hostname: &str,
        addr_type: AddrType,
    ) -> Result<IpAddr, Self::Error> {
        let accept_v4 = addr_type != AddrType::IPv6;
        let accept_v6 = addr_type != AddrType::IPv4;

        // We don't need a port, but the interface of to_socket_addrs (like getaddrinfo) insists on
        // ports being around.
        let fake_port = 1234;

        for addr in (hostname, fake_port).to_socket_addrs().await? {
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

        Err(Self::Error::new(std::io::ErrorKind::NotFound, NotFound))
    }

    async fn get_host_by_address(
        &self,
        addr: IpAddr,
        result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        let fakesocketaddr =
            std::net::SocketAddr::new(crate::conversion::IpAddr::from(addr).into(), 1234);

        let (name, _service) =
            async_std::task::spawn_blocking(move || dns_lookup::getnameinfo(&fakesocketaddr, 0))
                .await?;

        if name.parse::<std::net::IpAddr>().is_ok() {
            // embedded_nal requires a host name to be returned and is not content with stringified
            // IP addresses
            return Err(Self::Error::new(std::io::ErrorKind::NotFound, NotFound));
        }

        if let Some(result) = result.get_mut(..name.len()) {
            result.copy_from_slice(name.as_bytes());
            Ok(result.len())
        } else {
            Err(Self::Error::new(std::io::ErrorKind::OutOfMemory, TooLong))
        }
    }
}
