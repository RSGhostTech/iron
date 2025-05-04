use crate::web::functions::version_into_bytes;
use http::{StatusCode, Version};
use std::collections::{BTreeMap, HashMap};
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

    pub(super) fn decide_keys_is_valuable(sub: &ClientRequest, par: &ClientRequest) -> bool {
        match (sub, par) {
            (ClientRequest::Path(ss), ClientRequest::Path(ps)) => ss == ps,
            (ClientRequest::Keys(ss, sv), ClientRequest::Keys(ps, pv)) => {
                if ss != ps || sv.len() > pv.len() || sv.is_empty() {
                    //path不匹配，或者子集大于父集长度，或者长度为0(无效)需要做出default回应
                    false
                } else {
                    //将父集采用HashSet，然后再比较
                    let ph = pv.iter().map(|(k, _)| k).collect::<HashSet<_>>();
                    sv.iter().map(|(k, _)| k).all(|k| ph.contains(&k))
                }
            }
            (_, _) => false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum ClientRequest {
    Path(String),
    Keys(String, Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub enum Resource {
    LocalFile(String),
    Data(Vec<u8>),
}

impl Resource {
    pub async fn into_bytes(self) -> io::Result<Vec<u8>> {
        match self {
            Resource::LocalFile(path) => Ok(functions::read_local_file(&path).await?),
            Resource::Data(data) => Ok(data),
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
        WillResponse {
            data,
            headers: HashMap::new(),
            version: Version::HTTP_11,
            state: StatusCode::OK,
        }
    }

    #[inline]
    pub fn add_content_length(&mut self) {
        self.headers
            .insert("Content-Length".to_string(), format!("{}", self.data.len()));
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
        //预分配vec，减少性能开销
        let mut bytes = Vec::with_capacity(64 + self.headers.len() * 64 + self.data.len());

        //硬编码Version可以避免Version的String创建、UTF8检查，直接使用字节
        bytes.extend(version_into_bytes(self.version));

        bytes.push(b' ');

        //硬编码StateCode
        let code = self.state.as_u16();
        bytes.push(b'0' + (code / 100) as u8);
        bytes.push(b'0' + ((code % 100) / 10) as u8);
        bytes.push(b'0' + (code % 10) as u8);

        bytes.push(b' ');

        //硬编码reason
        bytes.extend_from_slice(self.state.canonical_reason().unwrap().as_bytes());

        bytes.extend_from_slice(b"\r\n");

        for (key, value) in self.headers {
            bytes.extend_from_slice(key.as_bytes());
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value.as_bytes());
            bytes.extend_from_slice(b"\r\n");
        }

        bytes.extend_from_slice(b"\r\n");

        bytes.extend(self.data);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Router {
    inner: Resource,
}

impl Router {
    pub fn new(response: Resource) -> Self {
        Router { inner: response }
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
    routes: BTreeMap<ClientRequest, Router>,
    default: Router,
}
impl ServiceRouter {
    pub fn new(default: Router) -> Self {
        Self {
            routes: BTreeMap::new(),
            default,
        }
    }

    #[inline]
    pub fn add_router(&mut self, request: ClientRequest, router: Router) -> Option<Router> {
        self.routes.insert(request, router)
    }

    #[inline]
    pub fn route(&self, request: &ClientRequest) -> Router {
        for (pc, pr) in self.routes.iter() {
            if functions::decide_keys_is_valuable(request, pc) {
                return pr.clone();
            }
        }
        self.default.clone()
    }
}
