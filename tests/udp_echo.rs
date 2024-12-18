use embedded_nal::nb::block;
use std::net::SocketAddr;

fn echo(stack: &mut impl embedded_nal::UdpFullStack, addr: &str) {
    let addr: SocketAddr = addr.parse().unwrap();
    let mut servsock = stack.socket().unwrap();
    let mut clisock = stack.socket().unwrap();

    stack.bind(&mut servsock, addr.port()).unwrap();
    stack.connect(&mut clisock, addr).unwrap();

    block!(stack.send(&mut clisock, b"ping")).unwrap();
    let mut buffer = [0u8; 10];
    let (received, cliaddr) = block!(stack.receive(&mut servsock, &mut buffer)).unwrap();
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], b"ping");

    block!(stack.send_to(&mut servsock, cliaddr, b"pong")).unwrap();
    let mut buffer = [0u8; 10];
    let (received, _) = block!(stack.receive(&mut clisock, &mut buffer)).unwrap();
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], b"pong");
}

#[test]
fn std_echov4() {
    let mut stack = std_embedded_nal::Stack::default();
    echo(&mut stack, "127.0.0.1:2342");
}

#[test]
fn std_echov6() {
    let mut stack = std_embedded_nal::Stack::default();
    echo(&mut stack, "[::1]:4223");
}
