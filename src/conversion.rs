use embedded_nal::nb;
use std::io;
use std::net;

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

impl From<IpAddr> for net::IpAddr {
    fn from(s: IpAddr) -> net::IpAddr {
        s.0
    }
}
