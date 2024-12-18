use crate::conversion::to_nb;
use crate::SocketState;
use embedded_nal::nb;
use embedded_nal::{UdpClientStack, UdpFullStack};
use std::io::{self, Error};
use std::net::{self, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub struct UdpSocket {
    state: SocketState<net::UdpSocket, net::UdpSocket>,
}

impl UdpSocket {
    fn new() -> Self {
        Self {
            state: SocketState::new(),
        }
    }

    /// Return the raw file descriptor underlying the current socket.
    ///
    /// This is primarily intended for use with `select` style mechanisms: Any of the `nb` methods
    /// of the socket's traits, once returning [`nb::Error::WouldBlock`], will only make progress
    /// if data or buffer is available on that file descriptor.
    ///
    /// If this returns `None`, then the socket is still in a state where it doesn't even have an
    /// underlying operating system socket, and needs further operations ([UdpFullStack::bind] or
    /// [UdpClientStack::connect]) to be performed before it can be waited on. (Then again, a
    /// socket that doesn't return a raw file descriptor should never return `WouldBlock`). Being
    /// fallible, this is a method and not a trait implemntation of [std::os::unix::io::AsRawFd].
    #[cfg(any(unix, target_os = "wasi"))]
    pub fn as_raw_fd(&self) -> Option<std::os::unix::io::RawFd> {
        use std::os::unix::io::AsRawFd;

        Some(self.state.get_any().ok()?.as_raw_fd())
    }
}

impl UdpClientStack for crate::Stack {
    type UdpSocket = UdpSocket;
    type Error = Error;

    fn socket(&mut self) -> Result<Self::UdpSocket, Self::Error> {
        Ok(UdpSocket::new())
    }

    fn connect(&mut self, socket: &mut Self::UdpSocket, remote: SocketAddr) -> std::io::Result<()> {
        let any = match remote {
            SocketAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            SocketAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
        };

        let sock = net::UdpSocket::bind(any)?;

        sock.set_nonblocking(true)?;

        sock.connect(remote)?;
        socket.state = SocketState::Connected(sock);
        Ok(())
    }

    fn send(&mut self, socket: &mut Self::UdpSocket, buffer: &[u8]) -> nb::Result<(), Self::Error> {
        let sock = socket.state.get_running()?;
        sock.send(buffer).map(drop).map_err(to_nb)
    }

    fn receive(
        &mut self,
        socket: &mut Self::UdpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(usize, SocketAddr), Self::Error> {
        let sock = socket.state.get_any_mut()?;
        sock.recv_from(buffer).map_err(to_nb)
    }

    fn close(&mut self, _: Self::UdpSocket) -> io::Result<()> {
        // No-op: Socket gets closed when it is freed
        //
        // Could wrap it in an Option, but really that'll only make things messier; users will
        // probably drop the socket anyway after closing, and can't expect it to be usable with
        // this API.
        Ok(())
    }
}

impl UdpFullStack for crate::Stack {
    fn bind(&mut self, socket: &mut UdpSocket, port: u16) -> Result<(), Error> {
        let anyaddressthisport = net::SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

        let sock = net::UdpSocket::bind(anyaddressthisport)?;

        sock.set_nonblocking(true)?;

        socket.state = SocketState::Bound(sock);
        Ok(())
    }
    fn send_to(
        &mut self,
        socket: &mut UdpSocket,
        remote: SocketAddr,
        buffer: &[u8],
    ) -> Result<(), nb::Error<Error>> {
        let sock = socket.state.get_bound()?;
        sock.send_to(buffer, remote).map(drop).map_err(to_nb)
    }
}
