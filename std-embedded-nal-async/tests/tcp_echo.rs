use embedded_nal_async::{SocketAddr, TcpConnect};

// embedded_nal_async has no TCP server functionality, running one on blocking standard.
fn start_server(addr: &str) -> std::net::TcpListener {
    std::net::TcpListener::bind(addr).unwrap()
}

fn run_server(servsock: std::net::TcpListener) {
    use std::io::{Read, Write};

    let mut servsock = servsock.incoming().next().unwrap().unwrap();

    let mut buffer = [0u8; 4];
    servsock.read_exact(&mut buffer).unwrap();
    assert_eq!(&buffer, b"ping");

    servsock.write_all(b"pong").unwrap();
}

async fn echo(stack: &mut impl TcpConnect, addr: &'static str) {
    use embedded_io_async::{Read, Write};

    let server = async_std::task::spawn_blocking(|| start_server(addr)).await;
    let server = async_std::task::spawn_blocking(move || run_server(server));

    let addr: SocketAddr = addr.parse().unwrap();

    let mut clisock = stack.connect(addr).await.unwrap();

    clisock.write_all(b"ping").await.unwrap();

    let mut buffer = [0u8; 4];
    clisock.read_exact(&mut buffer).await.unwrap();
    assert_eq!(&buffer, b"pong");

    drop(server);
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
