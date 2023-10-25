use crate::conversion::SocketAddr;
use crate::SocketState;
use embedded_nal::nb;
use embedded_nal::{TcpClientStack, TcpFullStack};
use std::io::{Error, Read, Write};
use std::net::{self, IpAddr, Ipv6Addr, TcpListener, TcpStream};

#[derive(Debug)]
pub struct TcpError(pub Error);

impl From<Error> for TcpError {
    fn from(e: Error) -> Self {
        Self(e)
    }
}

impl TcpError {
    fn to_nb(e: Error) -> nb::Error<Self> {
        use std::io::ErrorKind::{TimedOut, WouldBlock};
        match e.kind() {
            WouldBlock | TimedOut => nb::Error::WouldBlock,
            _ => nb::Error::Other(Self(e)),
        }
    }
}

impl embedded_nal::TcpError for TcpError {
    fn kind(&self) -> embedded_nal::TcpErrorKind {
        match self.0.kind() {
            std::io::ErrorKind::BrokenPipe => embedded_nal::TcpErrorKind::PipeClosed,
            _ => embedded_nal::TcpErrorKind::Other,
        }
    }
}

pub struct TcpSocket {
    state: SocketState<TcpStream, TcpListener>,
}

impl TcpSocket {
    fn new() -> Self {
        Self {
            state: SocketState::new(),
        }
    }

    fn connected(s: TcpStream) -> Self {
        Self {
            state: SocketState::Connected(s),
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

        match &self.state {
            SocketState::Connected(s) => Some(s.as_raw_fd()),
            SocketState::Bound(s) => Some(s.as_raw_fd()),
            SocketState::Building => None,
        }
    }
}

impl TcpClientStack for crate::Stack {
    type TcpSocket = TcpSocket;
    type Error = TcpError;

    fn socket(&mut self) -> Result<TcpSocket, Self::Error> {
        Ok(TcpSocket::new())
    }

    fn connect(
        &mut self,
        socket: &mut TcpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> nb::Result<(), Self::Error> {
        let soc = TcpStream::connect(SocketAddr::from(remote)).map_err(Self::Error::from)?;

        soc.set_nonblocking(true).map_err(Self::Error::from)?;

        socket.state = SocketState::Connected(soc);
        Ok(())
    }

    fn send(&mut self, socket: &mut TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let socket = socket.state.get_running().map_err(Self::Error::from)?;
        socket.write(buffer).map_err(Self::Error::to_nb)
    }

    fn receive(
        &mut self,
        socket: &mut TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        let socket = socket.state.get_running().map_err(Self::Error::from)?;
        socket.read(buffer).map_err(Self::Error::to_nb)
    }

    fn close(&mut self, _: TcpSocket) -> Result<(), Self::Error> {
        // No-op: Socket gets closed when it is freed
        //
        // Could wrap it in an Option, but really that'll only make things messier; users will
        // probably drop the socket anyway after closing, and can't expect it to be usable with
        // this API.
        Ok(())
    }
}

impl TcpFullStack for crate::Stack {
    fn bind(&mut self, socket: &mut TcpSocket, port: u16) -> Result<(), Self::Error> {
        let anyaddressthisport = net::SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

        let sock = TcpListener::bind(SocketAddr::from(anyaddressthisport))?;

        sock.set_nonblocking(true)?;

        socket.state = SocketState::Bound(sock);
        Ok(())
    }

    fn listen(&mut self, _: &mut TcpSocket) -> Result<(), Self::Error> {
        // Seems to be implied in listener creation
        Ok(())
    }

    fn accept(
        &mut self,
        socket: &mut TcpSocket,
    ) -> nb::Result<(TcpSocket, embedded_nal::SocketAddr), Self::Error> {
        let sock = socket.state.get_bound().map_err(Self::Error::from)?;
        sock.accept()
            .map_err(Self::Error::to_nb)
            .map(|(s, a)| (TcpSocket::connected(s), SocketAddr::from(a).into()))
    }
}

#[cfg(feature = "embedded-nal-tcpextensions")]
impl embedded_nal_tcpextensions::TcpExactStack for crate::Stack {
    // Arbitrary, but a) the std stack could allocate arbitrarily anyway, and b) this doesn't
    // read into the output buffer incompletely (which is what'd make buffering tricky)
    const RECVBUFLEN: usize = 4 * 1024 * 1024;
    const SENDBUFLEN: usize = 4 * 1024 * 1024;

    fn receive_exact(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<(), Self::Error> {
        let socket = socket.state.get_running()?;
        socket.read_exact(buffer).map_err(Self::Error::to_nb)
    }

    fn send_all(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> Result<(), nb::Error<Self::Error>> {
        let socket = socket.state.get_running()?;
        socket.write_all(buffer).map_err(Self::Error::to_nb)
    }
}
