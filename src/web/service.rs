use crate::web::client::HTTPClient;
use tokio::io;
use tokio::net::{TcpListener, ToSocketAddrs};

pub(super) mod functions {
    use crate::web::client::HTTPClient;
    use std::io::ErrorKind;
    use tokio::io;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpStream;

    pub(super) async fn doing_request(mut stream: TcpStream) -> io::Result<HTTPClient> {
        let mut buffer = [0; 4096];
        let len = stream.read(&mut buffer).await?;
        if let Some(client) = HTTPClient::new(stream, &buffer[..len]) {
            Ok(client)
        } else {
            Err(io::Error::new(
                ErrorKind::ConnectionAborted,
                "Because of bad http request,connect aborted",
            ))
        }
    }
}

pub struct HTTPService {
    listener: TcpListener,
}
impl HTTPService {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
        })
    }
    pub async fn accept(&mut self) -> io::Result<HTTPClient> {
        let (stream, _) = self.listener.accept().await?;
        functions::doing_request(stream).await
    }
}
