#![cfg(feature = "async")]

async fn echo(stack: &mut impl embedded_nal_async::UdpFullStack, addr: &str) {
    let addr: embedded_nal_async::SocketAddr = addr.parse().unwrap();
    let mut servsock = stack.socket().await.unwrap();
    let mut clisock = stack.socket().await.unwrap();

    stack.bind(&mut servsock, addr.port()).await.unwrap();
    stack.connect(&mut clisock, addr).await.unwrap();

    stack.send(&mut clisock, b"ping").await.unwrap();
    let mut buffer = [0u8; 10];
    let (received, cliaddr) = stack.receive(&mut servsock, &mut buffer).await.unwrap();
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], b"ping");

    stack.send_to(&mut servsock, cliaddr, b"pong").await.unwrap();
    let mut buffer = [0u8; 10];
    let (received, _) = stack.receive(&mut clisock, &mut buffer).await.unwrap();
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], b"pong");
}

#[test]
fn std_echov4() {
    let mut stack = std_embedded_nal::Stack::default();
    async_std::task::block_on(echo(&mut stack, "127.0.0.1:2342"));
}

#[test]
fn std_echov6() {
    let mut stack = std_embedded_nal::Stack::default();
    async_std::task::block_on(echo(&mut stack, "[::1]:4223"));
}
