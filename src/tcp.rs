use crate::conversion::{to_nb, SocketAddr};
use crate::SocketState;
use embedded_nal::nb;
use embedded_nal::{TcpClientStack, TcpFullStack};
use std::io::{self, Error, Read, Write};
use std::net::{self, TcpStream, TcpListener, IpAddr, Ipv6Addr};

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
            state: SocketState::Connected(s)
        }
    }
}

impl TcpClientStack for crate::Stack {
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn socket(&mut self) -> io::Result<TcpSocket> {
        Ok(TcpSocket::new())
    }

    fn connect(
        &mut self,
        socket: &mut TcpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> nb::Result<(), Self::Error> {
        let soc = TcpStream::connect(SocketAddr::from(remote))?;

        soc.set_nonblocking(true)?;

        socket.state = SocketState::Connected(soc);
        Ok(())
    }

    fn is_connected(&mut self, socket: &TcpSocket) -> io::Result<bool> {
        Ok(matches!(socket.state, SocketState::Connected(_)))
    }

    fn send(&mut self, socket: &mut TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let socket = socket.state.get_running()?;
        socket.write(buffer).map_err(to_nb)
    }

    fn receive(
        &mut self,
        socket: &mut TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        let socket = socket.state.get_running()?;
        socket.read(buffer).map_err(to_nb)
    }

    fn close(&mut self, _: TcpSocket) -> io::Result<()> {
        // No-op: Socket gets closed when it is freed
        //
        // Could wrap it in an Option, but really that'll only make things messier; users will
        // probably drop the socket anyway after closing, and can't expect it to be usable with
        // this API.
        Ok(())
    }
}

impl TcpFullStack for crate::Stack {
    fn bind(&mut self, socket: &mut TcpSocket, port: u16) -> Result<(), Error> {
        let anyaddressthisport = net::SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);

        let sock = TcpListener::bind(SocketAddr::from(anyaddressthisport))?;

        sock.set_nonblocking(true)?;

        socket.state = SocketState::Bound(sock);
        Ok(())
    }

    fn listen(&mut self, _: &mut TcpSocket) -> Result<(), Error> {
        // Seems to be implied in listener creation
        Ok(())
    }

    fn accept(&mut self, socket: &mut TcpSocket) -> nb::Result<(TcpSocket, embedded_nal::SocketAddr), Self::Error> {
        let sock = socket.state.get_bound()?;
        sock.accept()
            .map_err(to_nb)
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
        socket.read_exact(buffer).map_err(to_nb)
    }

    fn send_all(&mut self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> Result<(), nb::Error<Self::Error>> {
        let socket = socket.state.get_running()?;
        socket.write_all(buffer).map_err(to_nb)
    }
}
