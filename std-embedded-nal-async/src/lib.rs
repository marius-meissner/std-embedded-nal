//! This crate implements the [embedded-nal-async] network traits for operating systems that
//! support the standard library's network.
//!
//! As of now, only UDP sockets are implemented.
//!
//! All implementations use `std::io::Error` as their error type.
//!
//! [embedded-nal-async]: https://crates.io/crates/embedded-nal-async

mod conversion;
mod udp;

/// The operating system's network stack, implementing ``embedded_nal_async::UdpStack``.
///
/// The user may instantiate a stack using the `Stack::default()` function.
///
/// The stack can be cloned, as it is not a resource that needs any synchronization. This is not
/// made implicit as Copy, though (although there's not technical reason not to). That is to alert
/// users to the difficulties that'd arise when copying around a stack rather than using it through
/// some mechanism of synchronization.
#[derive(Clone, Default)]
pub struct Stack;
