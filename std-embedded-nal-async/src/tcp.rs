//! TCP implementation on the standard stack for embedded-nal-async

use crate::conversion;
use std::io::Error;

impl embedded_nal_async::TcpConnect for crate::Stack {
    type Error = Error;

    type Connection<'a> = TcpConnection;

    async fn connect<'a>(
        &'a self,
        addr: embedded_nal_async::SocketAddr,
    ) -> Result<Self::Connection<'a>, Error>
    where
        Self: 'a,
    {
        async_std::net::TcpStream::connect(async_std::net::SocketAddr::from(
            conversion::SocketAddr::from(addr),
        ))
        .await
        .map(TcpConnection)
    }
}

pub struct TcpConnection(async_std::net::TcpStream);

impl embedded_io_async::ErrorType for TcpConnection {
    type Error = Error;
}

impl embedded_io_async::Read for TcpConnection {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        use async_std::io::ReadExt;
        self.0.read(buffer).await
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), embedded_io_async::ReadExactError<Error>> {
        use async_std::io::ReadExt;
        self.0.read_exact(buffer).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::UnexpectedEof => embedded_io_async::ReadExactError::UnexpectedEof,
            _ => embedded_io_async::ReadExactError::Other(e),
        })
    }
}
impl embedded_io_async::Write for TcpConnection {
    async fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        use async_std::io::WriteExt;
        self.0.write(buffer).await
    }

    async fn flush(&mut self) -> Result<(), Error> {
        use async_std::io::WriteExt;
        self.0.flush().await
    }

    async fn write_all(&mut self, buffer: &[u8]) -> Result<(), Error> {
        use async_std::io::WriteExt;
        self.0.write_all(buffer).await
    }
}
