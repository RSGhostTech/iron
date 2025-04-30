use crate::web::metadata::HTTPMetadata;
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
}
