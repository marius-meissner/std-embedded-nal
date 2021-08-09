//! This crate implements the [embedded-nal] network traits for operating systems that support the
//! standard library's network.
//!
//! UDP and TCP sockets are plainly wrapped and should behave unsuspiciously.
//!
//! The DNS implementation is slightly incomplete, as the Rust standard library contains no
//! provisions to turn IP addresses back into hosts; that call thus fails unconditionally.
//!
//! All implementations use `std::io::Error` as their error type.
//!
//! [embedded-nal]: https://crates.io/crates/embedded-nal

mod conversion;
mod dns;
mod tcp;
mod udp;

/// The operating system's network stack, implementing ``embedded_nal::UdpStack`` and others.
///
/// The user may instantiate a stack using the `Stack::default()` function.
///
/// The stack can be cloned, as it is not a resource that needs any synchronization. This is not
/// made implicit as Copy, though (although there's not technical reason not to). That is to alert
/// users to the difficulties that'd arise when taking ownership of a stack rather than just using
/// it through a shared reference (which is generally possible in ``embedded_nal``).
#[derive(Clone, Default)]
pub struct Stack;

#[deprecated(note="Use Stack::default() instead.")]
pub static STACK: Stack = Stack;

/// An std::io::Error compatible error type returned when an operation is requested in the wrong
/// sequence (where the "right" is create a socket, connect, any receive/send, and possibly close).
#[derive(Debug)]
struct OutOfOrder;

impl std::fmt::Display for OutOfOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Out of order operations requested")
    }
}

impl std::error::Error for OutOfOrder {}

impl<T> Into<std::io::Result<T>> for OutOfOrder {
    fn into(self) -> std::io::Result<T> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            OutOfOrder,
        ))
    }
}

/// Socket
enum SocketState<T> {
    Building,
    Connected(T),
    Bound(T),
}

impl<T> SocketState<T> {
    fn new() -> Self {
        Self::Building
    }

    fn get_running(&mut self) -> std::io::Result<&mut T> {
        match self {
            SocketState::Connected(ref mut s) => Ok(s),
            _ => OutOfOrder.into(),
        }
    }

    fn get_bound(&mut self) -> std::io::Result<&mut T> {
        match self {
            SocketState::Bound(ref mut s) => Ok(s),
            _ => OutOfOrder.into(),
        }
    }

    fn get_any(&mut self) -> std::io::Result<&mut T> {
        match self {
            SocketState::Connected(ref mut s) => Ok(s),
            SocketState::Bound(ref mut s) => Ok(s),
            _ => OutOfOrder.into(),
        }
    }
}
