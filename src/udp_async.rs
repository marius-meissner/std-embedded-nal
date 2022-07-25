//! UDP implementation on the standard stack for embedded-nal-async
//!
//! This is an adjusted copy/paste from the [crate::udp] module.
//!
//! Futures may be implemeted with needless statefulness (they might have two inner .await points);
//! the author is not sure whether that should be improved (would it make the future zero-sized),
//! whether it can (as a rule of thumb, if it worked for nb it should support zero-sized futures)
//! or whether it even makes a difference after an LTO pass.

use crate::conversion::SocketAddr;
use crate::SocketState;
use std::io::Error;
use std::net::{self, IpAddr, Ipv4Addr, Ipv6Addr};

pub struct UdpSocket {
    state: SocketState<async_std::net::UdpSocket, async_std::net::UdpSocket>,
}

impl UdpSocket {
    fn new() -> Self {
        Self {
            state: SocketState::new(),
        }
    }

    // Not providing as_raw_fd: The only reason this should be here is to enable async, which here
    // is automatic.
}

impl embedded_nal_async::UdpClientStack for crate::Stack {
    type UdpSocket = UdpSocket;
    type Error = Error;
    type SocketFuture<'m> = impl std::future::Future<Output = Result<Self::UdpSocket, Self::Error>> where Self: 'm;
    type ConnectFuture<'m> = impl std::future::Future<Output = Result<(), Self::Error>> where Self: 'm;
    type SendFuture<'m> = impl std::future::Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    type ReceiveFuture<'m> = impl std::future::Future<Output = Result<(usize, embedded_nal::SocketAddr), Self::Error>> + 'm
    where
        Self: 'm;
    type CloseFuture<'m> = std::future::Ready<Result<(), Self::Error>> where Self: 'm;

    fn socket<'m>(&'m mut self) -> Self::SocketFuture<'m> {
        std::future::ready(Ok(UdpSocket::new()))
    }

    fn connect<'m>(
        &'m mut self,
        socket: &'m mut Self::UdpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> Self::ConnectFuture<'m> {
        async move {
            let any = match remote {
                embedded_nal::SocketAddr::V4(_) => {
                    net::SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
                }
                embedded_nal::SocketAddr::V6(_) => {
                    net::SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
                }
            };

            let sock = async_std::net::UdpSocket::bind(any).await?;

            use std::net::ToSocketAddrs;
            sock.connect(SocketAddr::from(remote).to_socket_addrs()?
                         .next()
                         .expect("Addresses converted from an embedded_nal address have exactly one socket address"))
                .await?;
            socket.state = SocketState::Connected(sock);
            Ok(())
        }
    }

    fn send<'m>(
        &'m mut self,
        socket: &'m mut Self::UdpSocket,
        buffer: &'m [u8],
    ) -> Self::SendFuture<'m> {
        async move {
            let sock = socket.state.get_running()?;
            sock.send(buffer).await.map(drop)
        }
    }

    fn receive<'m>(
        &'m mut self,
        socket: &'m mut Self::UdpSocket,
        buffer: &'m mut [u8],
    ) -> Self::ReceiveFuture<'m> {
        async move {
            let sock = socket.state.get_any_mut()?;
            sock.recv_from(buffer)
                .await
                .map(|(length, peer_addr)| (length, SocketAddr::from(peer_addr).into()))
        }
    }
    fn close<'m>(&'m mut self, _socket: Self::UdpSocket) -> Self::CloseFuture<'m> {
        // No-op: Socket gets closed when it is freed
        //
        // Could wrap it in an Option, but really that'll only make things messier; users will
        // probably drop the socket anyway after closing, and can't expect it to be usable with
        // this API.
        std::future::ready(Ok(()))
    }
}
