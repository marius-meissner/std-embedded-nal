/// A brutally oversimplified HTTP client that GETs / from localhost:80
use embedded_nal::nb::block;
use embedded_nal::{Dns, TcpClientStack};

fn run<S, E>(stack: &mut S) -> Result<(), E>
where
    E: core::fmt::Debug, // Might go away when MSRV goes up to 1.49, see https://github.com/rust-lang/rust/issues/80821
    S: TcpClientStack<Error = E> + Dns<Error = E>,
{
    let target = embedded_nal::SocketAddr::new(
        block!(stack.get_host_by_name("localhost", embedded_nal::AddrType::IPv6))?,
        80,
    );

    let mut sock = stack.socket()?;
    block!(stack.connect(&mut sock, target))?;
    block!(stack.send(&mut sock, b"GET / HTTP/1.0\r\nHostname: localhost\r\n\r\n"))?;

    let mut respbuf = [0; 1500];
    let resplen = block!(stack.receive(&mut sock, &mut respbuf))?;
    let response = &respbuf[..resplen];

    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

fn main() {
    let mut stack = std_embedded_nal::Stack::default();

    run(&mut stack).expect("Error running the main program")
}
