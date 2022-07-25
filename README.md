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

# Performance and non-blocking

The client main examples run use `nb::block!`,
which means that there is busy looping until an event arrives
(which is bad in clients and terrible in servers).

While the general async infrastructure of Rust is gaining traction,
embedded-nal does [not yet] support that,
and some use cases might never.

Finding when to retry will to some extent be application specific as long as async is not used.
The TCP example plainly blocks which is terrible in terms of performance.
The UDP example, on UNIX, uses `mio` to use the socket's exported raw file descriptor to wait until data is available.
This approach will need some more evaluation before it can be recommended;
chances are it will be overtaken by async work.

Until either of that is widely available,
users should be aware that `nb::block!` based solutions on `std` systems
(or anywhere, really)
have abysmal performance properties,
and should not used outside of experimentation and testing.

[not yet]: https://github.com/rust-embedded-community/embedded-nal/issues/6

# Maturity

This crate contains minimal working implementations the traits currently in embedded-nal.

# Minimum Supported Rust Version

This crate is build-tested on stable Rust 1.51.0.
That is largely following the embedded-nal MSRV.
It *might* compile with older versions but that may change at any time.

[embedded-nal]: https://crates.io/crates/embedded-nal
[linux-embedded-hal]: https://crates.io/crates/linux-embedded-hal
[embedded-hal]: https://crates.io/crates/embedded-hal
