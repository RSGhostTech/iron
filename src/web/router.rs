use crate::web::functions::version_into_string;
use http::{StatusCode, Version};
use std::collections::HashMap;
use tokio::io;

pub(super) mod functions {
    use crate::web::router::ClientRequest;
    use std::collections::HashSet;
    use tokio::fs::File;
    use tokio::io;

    pub(super) async fn read_local_file(path: &str) -> io::Result<Vec<u8>> {
        let mut reader = io::BufReader::new(File::open(path).await?);
        let mut buf = Vec::new();
        io::copy(&mut reader, &mut buf).await?;
        Ok(buf)
    }

    pub(super) fn decide_keys_is_valuable(
        sub: &ClientRequest,
        par: &ClientRequest,
    ) -> Option<bool> {
        match sub {
            ClientRequest::Path(_) => None,
            ClientRequest::Keys(sp, sv) => {
                if let ClientRequest::Keys(pp, pv) = par {
                    if pp == sp {
                        let sv = sv.iter().collect::<HashSet<_>>();
                        let pv = pv.iter().collect::<HashSet<_>>();
                        for i in pv {
                            if !sv.contains(i) {
                                return Some(false);
                            }
                        }
                        Some(true)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ClientRequest {
    Path(String),
    Keys(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub enum ServerResponse {
    LocalFile(String),
    Data(Vec<u8>),
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
pub struct WillResponse {
    data: Vec<u8>,
    headers: HashMap<String, String>,
    version: Version,
    state: StatusCode,
}

impl WillResponse {
    pub fn new(data: Vec<u8>) -> Self {
        WillResponse { data, headers: HashMap::new(), version: Version::HTTP_11, state: StatusCode::OK }
    }

    #[inline]
    pub fn add_content_length(&mut self) {
        self.headers.insert("Content-Length".to_string(), format!("{}", self.data.len()));
    }

    #[inline]
    pub fn add_ky(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    #[inline]
    pub fn state(&mut self, state: StatusCode) {
        self.state = state;
    }

    #[inline]
    pub fn version(&mut self, version: Version) {
        self.version = version;
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        let b1 = format!(
            "{} {} {}\r\n",
            version_into_string(self.version),
            self.state.as_u16(),
            self.state.canonical_reason().unwrap()
        ).into_bytes();

        let mut b2 = Vec::new();
        for (key, value) in &self.headers {
            b2.push(format!("{}: {}\r\n", key, value).into_bytes());
        }
        b2.push(Vec::from("\r\n"));
        let b2 = b2.concat();

        [b1, b2, self.data].concat()
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

    pub async fn response(&self) -> io::Result<WillResponse> {
        Ok(WillResponse::new(self.inner.clone().into_bytes().await?))
    }

    pub fn route_to(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct ServiceRouter {
    routes: Vec<(ClientRequest, Route)>,
    default: Route,
}
impl ServiceRouter {
    pub fn new(default: Route) -> Self {
        Self {
            routes: Vec::new(),
            default,
        }
    }

    #[inline]
    pub fn add_route(self, request: ClientRequest, route: Route) -> Self {
        let mut hash = self.routes.into_iter().collect::<HashMap<_, _>>();
        hash.insert(request, route);
        Self {
            routes: hash.into_iter().collect(),
            ..self
        }
    }

    #[inline]
    pub fn route(&self, request: &ClientRequest) -> Route {
        for (pc, pr) in self.routes.iter() {
            if let Some(true) = functions::decide_keys_is_valuable(request, pc) {
                return pr.clone();
            }
        }
        self.default.clone()
    }
}
