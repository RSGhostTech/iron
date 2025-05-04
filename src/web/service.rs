use crate::web::client::HTTPClient;
use crate::web::router::{Route, ServiceRouter};
use std::sync::Arc;
use tokio::io;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::runtime::Runtime;

pub(super) mod functions {
    use crate::web::client::HTTPClient;
    use crate::web::metadata::HTTPMetadata;
    use crate::web::router::ClientRequest;
    use std::io::ErrorKind;
    use tokio::io;
    use tokio::net::TcpStream;

    #[inline]
    pub(super) async fn accept(mut stream: TcpStream) -> io::Result<HTTPClient> {
        let mut buf = Vec::new();
        io::copy(&mut stream, &mut buf).await?;

        HTTPClient::new(stream, &buf).ok_or_else(|| {
            io::Error::new(
                ErrorKind::ConnectionAborted,
                "Because of bad http request,connect aborted",
            )
        })
    }

    #[inline]
    pub(super) fn get_client_request(metadata: &HTTPMetadata) -> ClientRequest {
        let path = metadata.path.clone();
        let keys = (!metadata.kv.is_empty()).then(|| {
            let mut temp = Vec::with_capacity(metadata.kv.len());
            metadata.kv.keys().for_each(|k| temp.push(k.clone()));
            temp
        });

        match keys {
            Some(k) => ClientRequest::Keys(path, k),
            None => ClientRequest::Path(path),
        }
    }
}

pub struct HTTPService {
    listener: TcpListener,
    router: ServiceRouter,
    runtime: Runtime,
}
impl HTTPService {
    pub async fn new<A: ToSocketAddrs>(
        addr: A,
        runtime: Runtime,
        default_route: Route,
    ) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
            router: ServiceRouter::new(default_route),
            runtime,
        })
    }
    pub async fn accept(&mut self) -> io::Result<HTTPClient> {
        let (stream, _) = self.listener.accept().await?;
        functions::accept(stream).await
    }

    pub async fn serve(mut self) -> ! {
        loop {
            let client = match self.accept().await {
                Ok(client) => client,
                Err(err) => {
                    log::error!("{}", err.to_string());
                    continue;
                }
            };

            let service_route = self.router.clone();
            self.runtime.spawn(async move {
                let client = client;
                let request = functions::get_client_request(client.metadata());
                let route = service_route.route(&request);
                let metadata = client.metadata().clone();
                let socket = client.addr();
                match client.response(&route).await {
                    Ok(mut responding) => {
                        responding.mut_will_response().add_content_length();
                        if let Err(err) = responding.send().await {
                            log::error!("{}", err.to_string());
                        } else {
                            log::info!(
                                "Has routed {} -> {} ({})",
                                metadata.path,
                                route.route_to(),
                                socket.to_string()
                            );
                        }
                    }
                    Err(err) => {
                        log::error!("{}", err.to_string());
                    }
                }
            });
        }
    }
}
