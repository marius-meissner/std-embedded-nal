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

mod udp;
mod tcp;
mod dns;
mod conversion;

/// The operating system's network stack, implementing ``embedded_nal::UdpStack`` and others.
///
/// This is most easily accessed using the static ``STACK`` instance.
///
/// The stack can be cloned, as it is not a resource that needs any synchronization. This is not
/// made implicit as Copy, though (although there's not technical reason not to). That is to alert
/// users to the difficulties that'd arise when taking ownership of a stack rather than just using
/// it through a shared reference (which is generally possible in ``embedded_nal``).
#[derive(Clone)]
pub struct Stack;

pub static STACK: Stack = Stack;
