# `std-embedded-nal-async`

This crate implements the [embedded-nal-async] network traits for operating systems that support the standard library's network.
.

In that, it is to embedded-nal what [linux-embedded-hal] is to [embedded-hal]:
A way to use libraries written for the bare-metal embedded world on Linux.
(Just that network interfaces are better standardized than hardware access, so it should work on any POSIX system).

# Usage

As the operating system's network stack is always available,
it can be instanciated and used at any time without need for synchronization, roughly like this:

```rust
use std_embedded_nal_async::Stack;

let message = [0x50, 0x01, 0x00, 0x00];

let mut stack = Stack::default();
let (_local, mut sock) = stack.connect("[::1]:5683".parse()?).await?;
sock.send(&message).await?;
```

See the examples for full and working versions.

# Portability

Unlike with the blocking traits,
design decisions taken for the async traits ensure that the local system's UDP address is always available,
which is essential for many UDP based services --
but the standard library does not make such features available.

They are available in a standardized form on POSIX systems,
which is why (unlike [std-embedded-nal]),
this crate depends on [nix] to provide the `recvmsg()` system call.

Adding support for Windows,
whose crate [indicates support for an equivalent operation],
should be straightforward for developers versed in that platform.

# Implementation choices w/rt IPv4

There are two ways of using IPv4 on most modern UNIX systems:
through IPv4 sockets,
and through IPv6 sockets using V4MAPPED addresses.

This library currently implements the former,
for the benefit of FreeBSD systems on which no mapping mechanism is implemented,
and because the standard library's addresses make using that pattern a bit easier
(being an enum over V4/V6 addresses, as opposed to being V6 addresses which may be V4-mapped).
As this avenue incurs some code duplication and maintenance overhead
(in particular, it requires going through a possibly platform specific pktinfo struct rather than the `struct in6_pktinfo` from [RFC 3542]),
that choice is considered an implementation detail,
and the library may switch to using V4-mapped addreses at a later point
(or introduce features to use either).

[embedded-nal-async]: https://crates.io/crates/embedded-nal-async
[linux-embedded-hal]: https://crates.io/crates/linux-embedded-hal
[embedded-hal]: https://crates.io/crates/embedded-hal
[std-embedded-nal]: https://crates.io/crates/std-embedded-nal
[nix]: https://crates.io/crates/nix
[indicates support for an equivalent operation]: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Networking/WinSock/struct.IN6_PKTINFO.html
[RFC 3542]: https://www.rfc-editor.org/rfc/rfc3542
