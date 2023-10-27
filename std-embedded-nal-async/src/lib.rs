//! This crate implements the [embedded-nal-async] network traits for operating systems that
//! support the standard library's network.
//!
//! As of now, only UDP sockets are implemented.
//!
//! All implementations use `std::io::Error` as their error type.
//!
//! [embedded-nal-async]: https://crates.io/crates/embedded-nal-async
//!
//! # Caveats
//!
//! ## Portability
//!
//! The current version uses the [nix] crate to get `IP_PKTINFO` / `IPV6_PKTINFO` from received
//! packets as the UDP trait requires. This is only portable within POSIX systems, where the
//! [`recvmsg` call](https://www.man7.org/linux/man-pages/man3/recvmsg.3p.html) is provided. The
//! UniquelyBound version works without that restriction, but so far there has not been a need to
//! run this on non-POSIX systems.
//!
//! ## UDP uniquely bound: excessive lengths
//!
//! Received messages whose length exceeds the provided buffer are not yet reported correctly for
//! UDP's uniquely bound sockets. (They work as they shoudl for the multiply bound sockets, which
//! depend on `recvmsg`, which allows detecting the overly long messages).
//!
//! ## Excessive lifetimes of receive buffers
//!
//! As required by the [embedded_nal_async] APIs, buffers are provided through exclusive references
//! that are pinned for th eduration of the Future. The receive functions each have only one await
//! point, so it would suffice if the buffers were provided just for the time of the poll calls on
//! the future (as it would be done on `nb`); it has yet to be evaluated whether (in an application
//! that uses up the buffers before it waits again) this makes an acutal difference when running on
//! full link time optimization.

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
