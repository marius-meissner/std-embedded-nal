//! A brutally oversimplified CoAP client that GETs /.well-known/core from localhost:5683
//!
//! A slightly more elaborate version illustrates how the socket's `as_raw_fd()` can be used to
//! start reading at the one time when it will succeed.

use embedded_nal::nb::block;
use embedded_nal::{Dns, UdpClientStack};
use std::net::SocketAddr;

pub fn run<S, E>(stack: &mut S) -> Result<(), E>
where
    E: core::fmt::Debug, // Might go away when MSRV goes up to 1.49, see https://github.com/rust-lang/rust/issues/80821
    S: UdpClientStack<Error = E> + Dns<Error = E>,
{
    let target = SocketAddr::new(
        block!(stack.get_host_by_name("localhost", embedded_nal::AddrType::IPv6))?,
        5683,
    );

    let mut sock = stack.socket()?;
    stack.connect(&mut sock, target)?;
    // Data, V1 NON no token, GET, message ID 0x0000, 2x Uri-Path
    block!(stack.send(&mut sock, b"\x50\x01\0\0\xbb.well-known\x04core"))?;

    let mut respbuf = [0; 1500];
    let (resplen, _) = block!(stack.receive(&mut sock, &mut respbuf))?;
    let response = &respbuf[..resplen];

    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

/// Like run, but rather than doing the only thing possible with plain `nb` and block, use that our
/// sockets have file descriptors and block at the OS level.
fn run_with_unix(stack: &mut std_embedded_nal::Stack) -> Result<(), std::io::Error> {
    let target = SocketAddr::new(
        block!(stack.get_host_by_name("localhost", embedded_nal::AddrType::IPv6))?,
        5683,
    );

    let mut sock = stack.socket()?;
    stack.connect(&mut sock, target)?;

    let fd = sock
        .as_raw_fd()
        .expect("Connected socket should already have an FD");
    let mut poll = mio::Poll::new()?;
    poll.registry().register(
        &mut mio::unix::SourceFd(&fd),
        mio::Token(0),
        mio::Interest::READABLE,
    )?;
    let mut events = mio::Events::with_capacity(1);

    // Data, V1 NON no token, GET, message ID 0x0000, 2x Uri-Path
    //
    // Not bothering with mio setup -- sending on UDP is practically instant anyway, making block!
    // a formality.
    block!(stack.send(&mut sock, b"\x50\x01\0\0\xbb.well-known\x04core"))?;

    poll.poll(&mut events, None)?;

    let mut respbuf = [0; 1500];
    let (resplen, _) = stack
        .receive(&mut sock, &mut respbuf)
        .map_err(|e| match e {
            embedded_nal::nb::Error::Other(o) => o,
            embedded_nal::nb::Error::WouldBlock => unreachable!("Polling said this could be read."),
        })?;
    let response = &respbuf[..resplen];

    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

fn main() {
    let mut stack = std_embedded_nal::Stack::default();

    #[cfg(unix)]
    run_with_unix(&mut stack).expect("Error running the main program");

    #[cfg(not(unix))]
    run(&mut stack).expect("Error running the main program");
}
