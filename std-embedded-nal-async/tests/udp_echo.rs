use embedded_nal_async::{ConnectedUdp, SocketAddr, UdpStack, UnconnectedUdp};

async fn echo(stack: &mut impl UdpStack, addr: &str) {
    let addr: SocketAddr = addr.parse().unwrap();

    let mut servsock = stack.bind_multiple(addr).await.unwrap();
    let (cli_local, mut clisock) = stack.connect(addr).await.unwrap();

    clisock.send(b"ping").await.unwrap();
    let mut buffer = [0u8; 10];
    let (received, servaddr, server_cliaddr) = servsock.receive_into(&mut buffer).await.unwrap();
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], b"ping");
    assert_eq!(
        server_cliaddr, cli_local,
        "Client local and server remote address differ; NAT on loopback??"
    );

    servsock
        .send(servaddr, server_cliaddr, b"pong")
        .await
        .unwrap();
    let mut buffer = [0u8; 10];
    let received = clisock.receive_into(&mut buffer).await.unwrap();
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], b"pong");
}

#[test]
fn std_echov4() {
    let mut stack = std_embedded_nal_async::Stack::default();
    async_std::task::block_on(echo(&mut stack, "127.0.0.1:2342"));
}

#[test]
fn std_echov6() {
    let mut stack = std_embedded_nal_async::Stack::default();
    async_std::task::block_on(echo(&mut stack, "[::1]:4223"));
}
