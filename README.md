# `std-embedded-nal`

This crate implements the [embedded-nal] network traits for operating systems that support the standard library's network.
.

In that, it is to embedded-nal what [linux-embedded-hal] is to [embedded-hal]:
A way to use libraries written for the bare-metal embedded world on Linux.
(Just that network interfaces are better standardized than hardware access, so it should work on any system).

# Usage

As the operating system's network stack is always available,
it can be referenced at any time, roughly like this:

```rust
use embedded_nal::nb::block;
use std_embedded_nal::STACK;
use embedded_nal::UdpClient;

let message = [0x50, 0x01, 0x00, 0x00];

let mut socket = STACK.socket()?;
block!(STACK.connect(&mut socket, "127.0.0.1:5683".parse()?)?);
block!(STACK.send(&mut socket, &message)?);
```

See the CoAP and HTTP examples for full and working versions.

# Maturity

This crate contains minimal working implementations of the traits currently in embedded-nal.

# Minimum Supported Rust Version

This crate is build-tested on stable Rust 1.36.0.
It *might* compile with older versions but that may change at any time.

[embedded-nal]: https://crates.io/crates/embedded-nal
[linux-embedded-hal]: https://crates.io/crates/linux-embedded-hal
[embedded-hal]: https://crates.io/crates/embedded-hal
