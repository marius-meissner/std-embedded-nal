/// A brutally oversimplified HTTP client that GETs / from localhost:80

use embedded_nal::nb::block;

fn run<S: embedded_nal::TcpStack + embedded_nal::Dns>(stack: &S) -> Result<(), <S as embedded_nal::TcpStack>::Error>
where
    <S as embedded_nal::TcpStack>::Error: std::convert::From<<S as embedded_nal::Dns>::Error>
{
    let target = embedded_nal::SocketAddr::new(stack.gethostbyname("localhost", embedded_nal::AddrType::IPv6)?, 80);

    let sock = stack.open(embedded_nal::Mode::Blocking)?;
    let mut sock = stack.connect(sock, target)?;
    // It's opened in blocking mode, so we're not really expecting WouldBlock, but this gets us rid
    // of the additional error type and allows `?` returns.
    block!(stack.write(&mut sock, b"GET / HTTP/1.0\r\nHostname: localhost\r\n\r\n"))?;

    let mut respbuf = [0; 1500];
    let resplen = block!(stack.read(&mut sock, &mut respbuf))?;
    let response = &respbuf[..resplen];

    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

fn main() {
    let stack = &std_embedded_nal::STACK;

    run(stack).expect("Error running the main program")
}
