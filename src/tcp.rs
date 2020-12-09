use crate::conversion::{to_nb, OutOfOrder, SocketAddr};
use embedded_nal::nb;
use embedded_nal::{Mode, TcpStack};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

enum SocketState {
    Building(Mode),
    Running(TcpStream),
}

pub struct TcpSocket {
    state: SocketState,
}

impl TcpSocket {
    fn get_running(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.state {
            SocketState::Running(ref mut s) => Ok(s),
            _ => Err(Error::new(ErrorKind::NotConnected, OutOfOrder)),
        }
    }

    fn get_mode(&self) -> io::Result<&Mode> {
        match &self.state {
            SocketState::Building(m) => Ok(&m),
            _ => Err(Error::new(ErrorKind::Other, OutOfOrder)),
        }
    }
}

impl TcpStack for crate::Stack {
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn open(&self, mode: Mode) -> io::Result<TcpSocket> {
        Ok(TcpSocket {
            state: SocketState::Building(mode),
        })
    }

    fn connect(
        &self,
        socket: TcpSocket,
        remote: embedded_nal::SocketAddr,
    ) -> io::Result<TcpSocket> {
        let mode = socket.get_mode()?;

        let socket = TcpStream::connect(SocketAddr::from(remote))?;

        match mode {
            Mode::NonBlocking => {
                socket.set_nonblocking(true)?;
            }
            Mode::Blocking => {
                socket.set_nonblocking(false)?;
                socket.set_read_timeout(None)?;
                socket.set_write_timeout(None)?;
            }
            Mode::Timeout(millis) => {
                socket.set_nonblocking(false)?;
                socket.set_read_timeout(Some(Duration::from_millis((*millis).into())))?;
                socket.set_write_timeout(Some(Duration::from_millis((*millis).into())))?;
            }
        }

        Ok(TcpSocket {
            state: SocketState::Running(socket),
        })
    }

    fn is_connected(&self, socket: &TcpSocket) -> io::Result<bool> {
        Ok(socket.get_mode().is_err())
    }

    fn write(&self, socket: &mut TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let socket = socket.get_running()?;
        socket.write(buffer).map_err(to_nb)
    }

    fn read(&self, socket: &mut TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        let socket = socket.get_running()?;
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
