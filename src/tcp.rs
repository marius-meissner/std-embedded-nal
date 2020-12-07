use crate::conversion::{to_nb, SocketAddr};
use crate::SocketState;
use embedded_nal::nb;
use embedded_nal::TcpClient;
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

impl TcpClient for crate::Stack {
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn socket(&self) -> io::Result<TcpSocket> {
        Ok(TcpSocket::new())
    }

    fn connect(
        &self,
        socket: &mut TcpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> nb::Result<(), Self::Error> {
        let soc = TcpStream::connect(SocketAddr::from(remote))?;

        soc.set_nonblocking(true)?;

        socket.state = SocketState::Running(soc);
        Ok(())
    }

    fn is_connected(&self, socket: &TcpSocket) -> io::Result<bool> {
        Ok(match socket.state {
            SocketState::Running(_) => true,
            _ => false,
        })
    }

    fn send(&self, socket: &mut TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let socket = socket.state.get_running()?;
        socket.write(buffer).map_err(to_nb)
    }

    fn receive(&self, socket: &mut TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        let socket = socket.state.get_running()?;
        socket.read(buffer).map_err(to_nb)
    }

    fn close(&self, _: TcpSocket) -> io::Result<()> {
        // No-op: Socket gets closed when it is freed
        //
        // Could wrap it in an Option, but really that'll only make things messier; users will
        // probably drop the socket anyway after closing, and can't expect it to be usable with
        // this API.
        Ok(())
    }
}
