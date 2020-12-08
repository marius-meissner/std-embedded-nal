use embedded_nal::nb;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::conversion::{to_nb, OutOfOrder, SocketAddr};

enum SocketState {
    Building(embedded_nal::Mode),
    Running(TcpStream),
}

pub struct TcpSocket {
    state: SocketState,
}

impl TcpSocket {
    fn get_running(&mut self) -> std::io::Result<&mut TcpStream> {
        match &mut self.state {
            SocketState::Running(ref mut s) => Ok(s),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                OutOfOrder,
            )),
        }
    }

    fn get_mode(&self) -> std::io::Result<&embedded_nal::Mode> {
        match &self.state {
            SocketState::Building(m) => Ok(&m),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, OutOfOrder)),
        }
    }
}

impl embedded_nal::TcpStack for crate::Stack {
    type TcpSocket = TcpSocket;
    type Error = std::io::Error;

    fn open(&self, mode: embedded_nal::Mode) -> std::io::Result<TcpSocket> {
        Ok(TcpSocket {
            state: SocketState::Building(mode),
        })
    }

    fn connect(
        &self,
        socket: TcpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> std::io::Result<TcpSocket> {
        let mode = socket.get_mode()?;

        let socket = TcpStream::connect(SocketAddr::from(remote))?;

        match mode {
            embedded_nal::Mode::NonBlocking => {
                socket.set_nonblocking(true)?;
            }
            embedded_nal::Mode::Blocking => {
                socket.set_nonblocking(false)?;
                socket.set_read_timeout(None)?;
                socket.set_write_timeout(None)?;
            }
            embedded_nal::Mode::Timeout(millis) => {
                socket.set_nonblocking(false)?;
                socket
                    .set_read_timeout(Some(std::time::Duration::from_millis((*millis).into())))?;
                socket
                    .set_write_timeout(Some(std::time::Duration::from_millis((*millis).into())))?;
            }
        }

        Ok(TcpSocket {
            state: SocketState::Running(socket),
        })
    }

    fn is_connected(&self, socket: &TcpSocket) -> Result<bool, Self::Error> {
        Ok(socket.get_mode().is_err())
    }

    fn write(
        &self,
        socket: &mut TcpSocket,
        buffer: &[u8],
    ) -> Result<usize, nb::Error<Self::Error>> {
        let socket = socket.get_running()?;
        socket.write(buffer).map_err(to_nb)
    }

    fn read(
        &self,
        socket: &mut TcpSocket,
        buffer: &mut [u8],
    ) -> Result<usize, nb::Error<Self::Error>> {
        let socket = socket.get_running()?;
        socket.read(buffer).map_err(to_nb)
    }

    fn close(&self, _: TcpSocket) -> Result<(), std::io::Error> {
        // No-op: Socket gets closed when it is freed
        //
        // Could wrap it in an Option, but really that'll only make things messier; users will
        // probably drop the socket anyway after closing, and can't expect it to be usable with
        // this API.
        Ok(())
    }
}
