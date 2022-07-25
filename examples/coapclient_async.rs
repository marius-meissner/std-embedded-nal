//! A brutally oversimplified CoAP client that GETs /.well-known/core from localhost:5683

use embedded_nal_async::UdpClientStack;

async fn run<S, E>(stack: &mut S) -> Result<(), E>
where
    E: core::fmt::Debug, // Might go away when MSRV goes up to 1.49, see https://github.com/rust-lang/rust/issues/80821
    S: UdpClientStack<Error = E>,
{
    let target = embedded_nal::SocketAddr::new(
        "::1".parse().unwrap(),
        5683,
    );

    let mut sock = stack.socket().await?;
    stack.connect(&mut sock, target).await?;
    // Data, V1 NON no token, GET, message ID 0x0000, 2x Uri-Path
    stack.send(&mut sock, b"\x50\x01\0\0\xbb.well-known\x04core").await?;

    let mut respbuf = [0; 1500];
    let (resplen, _) = stack.receive(&mut sock, &mut respbuf).await?;
    let response = &respbuf[..resplen];

    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

#[async_std::main]
async fn main() {
    let mut stack = std_embedded_nal::Stack::default();

    run(&mut stack).await.expect("Error running the main program");
}
