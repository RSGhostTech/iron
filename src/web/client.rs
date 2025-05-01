use crate::web::metadata::HTTPMetadata;
use crate::web::router::{Route, WillResponse};
use std::net::SocketAddr;
use tokio::io;
use tokio::io::AsyncWriteExt;
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

    pub async fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stream.write_all(bytes).await
    }

    pub async fn response(self, route: &Route) -> io::Result<Responding> {
        let b = route.response().await?;
        Ok(Responding::new(self, b))
    }
}

pub struct Responding {
    client: HTTPClient,
    will: WillResponse,
}

impl Responding {
    pub fn new(client: HTTPClient, will: WillResponse) -> Self {
        Self { client, will }
    }

    pub fn mut_will_response(&mut self) -> &mut WillResponse {
        &mut self.will
    }

    pub async fn send(mut self) -> io::Result<()> {
        let b = self.will.into_bytes();
        self.client.write_bytes(b.as_ref()).await
    }
}
