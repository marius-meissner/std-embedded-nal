//! This crate implements the [embedded-nal-async] network traits for operating systems that
//! support the standard library's network.
//!
//! As of now, only UDP sockets are implemented.
//!
//! All implementations use `std::io::Error` as their error type.
//!
//! [embedded-nal-async]: https://crates.io/crates/embedded-nal-async
//!
//! # Portability
//!
//! The current version uses the [nix] crate to get `IP_PKTINFO` / `IPV6_PKTINFO` from received
//! packets as the UDP trait requires. This is only portable within POSIX systems, where the
//! [`recvmsg` call](https://www.man7.org/linux/man-pages/man3/recvmsg.3p.html) is provided. The
//! UniquelyBound version works without that restriction, but so far there has not been a need to
//! run this on non-POSIX systems.

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
