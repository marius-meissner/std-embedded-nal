//! UDP implementation on the standard stack for embedded-nal-async
//!
//! This is an adjusted copy/paste from the [crate::udp] module.
//!
//! Futures may be implemeted with needless statefulness (they might have two inner .await points);
//! the author is not sure whether that should be improved (would it make the future zero-sized),
//! whether it can (as a rule of thumb, if it worked for nb it should support zero-sized futures)
//! or whether it even makes a difference after an LTO pass.
//!
//! Known bugs:
//! * Excessive lengths are not reported correctly except for the recvmsg version. This would be
//!   fixed by using recvmsg more widely.

use crate::conversion;
use crate::SocketState;
use core::future::Future;
use std::io::Error;

use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;

pub struct ConnectedSocket(async_std::net::UdpSocket);
pub struct UniquelyBoundSocket {
    socket: async_std::net::UdpSocket,
    // By storing this, we avoid the whole recvmsg hell, which we can because there's really only
    // one relevant address. (Alternatively, we could call `.local_addr()` over and over).
    bound_address: embedded_nal::SocketAddr,
}
pub struct MultiplyBoundSocket {
    socket: async_io::Async<std::net::UdpSocket>,
    // Storing this so we can return a full SocketAddr, even though pktinfo doesn't provide that
    // information
    port: u16,
}

impl embedded_nal_async::UdpStack for crate::Stack {
    type Error = Error;
    type Connected = ConnectedSocket;
    type UniquelyBound = UniquelyBoundSocket;
    type MultiplyBound = MultiplyBoundSocket;

    async fn connect_from(&self, local: embedded_nal::SocketAddr, remote: embedded_nal::SocketAddr) -> Result<(embedded_nal::SocketAddr, Self::Connected), Self::Error> {
        let sock = async_std::net::UdpSocket::bind(
            async_std::net::SocketAddr::from(conversion::SocketAddr::from(local))
            ).await?;

        sock.connect(
            async_std::net::SocketAddr::from(conversion::SocketAddr::from(remote))
            )
            .await?;

        let final_local = sock.local_addr()?;

        Ok((
                conversion::SocketAddr::from(final_local).into(),
                ConnectedSocket(sock)
                ))
    }

    async fn bind_single(&self, local: embedded_nal::SocketAddr) -> Result<(embedded_nal::SocketAddr, Self::UniquelyBound), Self::Error> {
        let sock = async_std::net::UdpSocket::bind(
            async_std::net::SocketAddr::from(conversion::SocketAddr::from(local))
            ).await?;

        let final_local = sock.local_addr()?;
        let final_local = conversion::SocketAddr::from(final_local).into();

        Ok((
                final_local,
                UniquelyBoundSocket { socket: sock, bound_address: final_local }
                ))
    }

    async fn bind_multiple(&self, local: embedded_nal::SocketAddr) -> Result<Self::MultiplyBound, Self::Error> {
        let sock = async_std::net::UdpSocket::bind(
            async_std::net::SocketAddr::from(conversion::SocketAddr::from(local))
            ).await?;

        // Due to https://github.com/async-rs/async-std/issues/1040 we have to leave the
        // friendly async_std territory and are on our own now.
        let sock: async_io::Async<std::net::UdpSocket> = unsafe { core::mem::transmute(sock) };
        let sock = sock.into_inner().unwrap();

        nix::sys::socket::setsockopt(
            sock.as_raw_fd(),
            nix::sys::socket::sockopt::Ipv6RecvPacketInfo,
            &true
            )?;

        let mut local_port = local.port();
        if local_port == 0 {
            local_port = sock.local_addr()?.port();
        }

        let sock = async_io::Async::new(sock)?;

        Ok(MultiplyBoundSocket { socket: sock, port: local_port })
    }
}

impl embedded_nal_async::ConnectedUdp for ConnectedSocket {
    type Error = Error;

    async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let sent_len = self.0.send(data).await?;
        assert!(sent_len == data.len(), "Datagram was not sent in a single operation");
        Ok(())
    }

    async fn receive_into(& mut self, buffer: & mut [u8]) -> Result<usize, Self::Error> {
        self.0.recv(buffer).await
    }
}

impl embedded_nal_async::UnconnectedUdp for UniquelyBoundSocket {
    type Error = Error;

    async fn send(&mut self, local: embedded_nal::SocketAddr, remote: embedded_nal::SocketAddr, data: &[u8]) -> Result<(), Self::Error> {
        debug_assert!(
            local == self.bound_address,
            "A socket created from bind_single must always provide its original local address (or the one returned from a receive) in send"
        );
        let remote: async_std::net::SocketAddr = conversion::SocketAddr::from(remote).into();
        let sent_len = self.socket.send_to(data, remote).await?;
        assert!(sent_len == data.len(), "Datagram was not sent in a single operation");
        Ok(())
    }

    async fn receive_into(&mut self, buffer: &mut [u8]) -> Result<(usize, embedded_nal::SocketAddr, embedded_nal::SocketAddr), Self::Error> {
        let (length, remote) = self.socket.recv_from(buffer).await?;
        let remote = conversion::SocketAddr::from(remote).into();
        Ok((length, self.bound_address, remote))
    }
}

impl embedded_nal_async::UnconnectedUdp for MultiplyBoundSocket {
    type Error = Error;

    async fn send(&mut self, local: embedded_nal::SocketAddr, remote: embedded_nal::SocketAddr, data: &[u8]) -> Result<(), Self::Error> {
        if local.port() != 0 {
            debug_assert_eq!(local.port(), self.port, "Packets can only be sent from the locally bound to port");
        }
        let remote: async_std::net::SocketAddr = conversion::SocketAddr::from(remote).into();
        // Taking this step on foot due to https://github.com/nix-rust/nix/issues/1754
        // FIXME v6-only
        let remote = match remote {
            async_std::net::SocketAddr::V6(a) => a,
            _ => panic!("Only IPv6 supported right now"),
        };
        let remote = std::net::SocketAddrV6::from(remote);
        let remote = nix::sys::socket::SockaddrIn6::from(remote);
        let local_pktinfo = conversion::IpAddr::from(local.ip()).into();
        self.socket.write_with(|s| {
            let sent_len = nix::sys::socket::sendmsg(
                s.as_raw_fd(),
                &[std::io::IoSlice::new(data)],
                // FIXME this ignores the IP part of the local address
                &[nix::sys::socket::ControlMessage::Ipv6PacketInfo(&local_pktinfo)],
                nix::sys::socket::MsgFlags::empty(),
                Some(&remote))?;
            assert!(sent_len == data.len(), "Datagram was not sent in a single operation");
            Ok(())
        }).await
    }

    async fn receive_into(&mut self, buffer: &mut [u8]) -> Result<(usize, embedded_nal::SocketAddr, embedded_nal::SocketAddr), Self::Error> {
        let (length, remote, local) = self.socket.read_with(|s| {
            let mut iov = [std::io::IoSliceMut::new(buffer)];
            let mut cmsg = nix::cmsg_space!(nix::libc::in6_pktinfo);
            let received = nix::sys::socket::recvmsg(
                s.as_raw_fd(),
                &mut iov,
                Some(&mut cmsg),
                nix::sys::socket::MsgFlags::MSG_TRUNC,
                )
                .map_err(Error::from)?;
            let local;
            if let Some(nix::sys::socket::ControlMessageOwned::Ipv6PacketInfo(pi)) = received.cmsgs().next() {
                local = embedded_nal::SocketAddr::new(
                    conversion::IpAddr::from(pi).into(),
                    self.port,
                    );
            } else {
                panic!("Operating system failed to send IPv6 packet info after acknowledging the socket option");
            }
            Ok((received.bytes, received.address, local))
        }).await?;

        let remote: nix::sys::socket::SockaddrStorage = remote
            .expect("recvmsg on UDP always returns a remote address");
        // Taking this step on foot due to https://github.com/nix-rust/nix/issues/1754
        let remote = remote.as_sockaddr_in6()
            .expect("Right now this is IPv6 only");
        let remote = std::net::SocketAddr::V6(
            std::net::SocketAddrV6::new(
                remote.ip(),
                remote.port(),
                remote.flowinfo(),
                remote.scope_id(),
                ));

        // We could probably shorten things by going more directly from SockaddrLike
        let remote = conversion::SocketAddr::from(remote).into();
        Ok((length, local, remote))
    }
}
