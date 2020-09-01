use std::net::{SocketAddr, IpAddr};
use embedded_nal::nb;

pub(crate) fn nal_to_std_ipaddr(input: embedded_nal::IpAddr) -> IpAddr {
    match input {
        embedded_nal::IpAddr::V4(i) => i.octets().into(),
        embedded_nal::IpAddr::V6(i) => i.octets().into(),
    }
}

pub(crate) fn nal_to_std_sockaddr(input: embedded_nal::SocketAddr) -> SocketAddr {
    (nal_to_std_ipaddr(input.ip()), input.port()).into()
}

pub(crate) fn std_to_nal_error(input: std::io::Error) -> nb::Error<std::io::Error> {
    match input.kind() {
        std::io::ErrorKind::WouldBlock => nb::Error::WouldBlock,
        // can be returned from read according to set_read-timeout
        std::io::ErrorKind::TimedOut => nb::Error::WouldBlock,
        _ => nb::Error::Other(input)
    }
}
