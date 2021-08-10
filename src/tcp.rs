use crate::conversion::{to_nb, SocketAddr};
use crate::SocketState;
use embedded_nal::nb;
use embedded_nal::TcpClientStack;
use std::io::{self, Error, Read, Write};
use std::net::TcpStream;

pub struct TcpSocket {
    state: SocketState<TcpStream>,
}

impl TcpSocket {
    fn new() -> Self {
        Self {
            state: SocketState::new(),
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
