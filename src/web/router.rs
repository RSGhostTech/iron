use std::collections::HashMap;
use tokio::io;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ClientRequest {
    Path(String),
    KeyValue(String, Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub enum ServerResponse {
    LocalFile(String),
    Data(Vec<u8>),
}

pub(super) mod functions {
    use tokio::fs::File;
    use tokio::io;

    pub(super) async fn read_local_file(path: &str) -> io::Result<Vec<u8>> {
        let mut reader = io::BufReader::new(File::open(path).await?);
        let mut buf = Vec::new();
        io::copy(&mut reader, &mut buf).await?;
        Ok(buf)
    }
}

impl ServerResponse {
    pub async fn into_bytes(self) -> io::Result<Vec<u8>> {
        match self {
            ServerResponse::LocalFile(path) => Ok(functions::read_local_file(&path).await?),
            ServerResponse::Data(data) => Ok(data),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    inner: ServerResponse,
}

impl Route {
    pub fn new(response: ServerResponse) -> Self {
        Route { inner: response }
    }

    pub async fn response(&self) -> io::Result<Vec<u8>> {
        Ok(self.inner.clone().into_bytes().await?)
    }

    pub fn route_to(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct ServiceRouter {
    routes: HashMap<ClientRequest, Route>,
    default: Route,
}
impl ServiceRouter {
    pub fn new(default: Route) -> Self {
        Self {
            routes: HashMap::new(),
            default,
        }
    }

    #[inline]
    pub fn add_route(mut self, request: ClientRequest, route: Route) -> Self {
        self.routes.insert(request, route);
        self
    }

    #[inline]
    pub fn route(&self, request: ClientRequest) -> Route {
        match self.routes.get(&request) {
            Some(route) => route.clone(),
            None => self.default.clone(),
        }
    }
}
