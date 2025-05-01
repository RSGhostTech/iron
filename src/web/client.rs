use crate::web::metadata::HTTPMetadata;
use crate::web::router::Route;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::TcpStream;

pub struct HTTPClient {
    metadata: HTTPMetadata,
    stream: TcpStream,
}

impl HTTPClient {
    pub fn new(stream: TcpStream, request: &[u8]) -> Option<HTTPClient> {
        Some(HTTPClient {
            stream,
            metadata: HTTPMetadata::new(request)?,
        })
    }

    pub fn metadata(&self) -> &HTTPMetadata {
        &self.metadata
    }

    pub fn addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    pub async fn response(&mut self, route: &Route) -> io::Result<()> {
        todo!()
    }
}
