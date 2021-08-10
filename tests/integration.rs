use embedded_nal::nb::block;
use embedded_nal::SocketAddr;

const PORT: u16 = 9876;

#[test]
fn udp_pingpong_self() {
    // Lots of unwrap here as things are known to work immediately.

    use embedded_nal::{UdpClientStack, UdpFullStack};

    let mut stack = std_embedded_nal::Stack::default();

    let mut server = stack.socket().unwrap();

    stack.bind(&mut server, PORT).unwrap();

    let mut client = stack.socket().unwrap();
    stack.connect(&mut client, SocketAddr::new("::1".parse().unwrap(), PORT)).unwrap();

    block!(stack.send(&mut client, b"ping")).unwrap();

    let mut buf = [0u8; 4];
    let req = block!(stack.receive(&mut server, &mut buf)).unwrap();
    assert_eq!(req.0, 4);
    assert_eq!(&buf, b"ping");

    block!(stack.send_to(&mut server, req.1, b"pong")).unwrap();
    let res = block!(stack.receive(&mut client, &mut buf)).unwrap();
    assert_eq!(res.0, 4);
    assert_eq!(&buf, b"pong");
}
