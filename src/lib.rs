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
pub struct Stack {
    // Ensure extensibility. Chances are we won't need it, but can still be relaxed easily.
    _private: ()
}

pub static STACK: Stack = Stack { _private: () };
