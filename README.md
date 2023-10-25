# `std-embedded-nal`

This crate implements the [embedded-nal] network traits for operating systems that support the standard library's network.
.

In that, it is to embedded-nal what [linux-embedded-hal] is to [embedded-hal]:
A way to use libraries written for the bare-metal embedded world on Linux.
(Just that network interfaces are better standardized than hardware access, so it should work on any system).

# Usage

As the operating system's network stack is always available,
it can be instanciated and used at any time without need for synchronization, roughly like this:

```rust
use embedded_nal::nb::block;
use std_embedded_nal::Stack;
use embedded_nal::UdpClient;

let message = [0x50, 0x01, 0x00, 0x00];

let mut stack = Stack::default();
let mut socket = stack.socket()?;
block!(stack.connect(&mut socket, "127.0.0.1:5683".parse()?)?);
block!(stack.send(&mut socket, &message)?);
```

See the CoAP and HTTP examples for full and working versions.

# Performance, non-blocking and async

When using the regular `embedded-nal` APIs,
the client main examples run use `nb::block!`,
which means that there is busy looping until an event arrives
(which is bad in clients and terrible in servers).

The general expectation with these APIs based on `nb` is that
users would know when to try again;
the UNIX version of the `coapclient` example illustrates how that would probably be done.
(The setup around `mio` and this library is relatively complex;
embedded implementations might get away with less code there.)

On nightly, and gated by the `async` feature,
the asynchronous implementations of the [embedded-nal-async] crate are available.

# Maturity

This crate contains minimal working implementations the traits currently in embedded-nal.

# Minimum Supported Rust Version

This crate is build-tested on stable Rust 1.53.0.
That is largely following the embedded-nal MSRV.
It *might* compile with older versions but that may change at any time.

[embedded-nal]: https://crates.io/crates/embedded-nal
[embedded-nal-async]: https://crates.io/crates/embedded-nal-async
[linux-embedded-hal]: https://crates.io/crates/linux-embedded-hal
[embedded-hal]: https://crates.io/crates/embedded-hal
