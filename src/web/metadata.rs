use crate::web::functions::version_from_str;
use http::{Method, Version};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HTTPMetadata {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub kv: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

pub(super) mod functions {
    use http::Method;
    use std::str::FromStr;

    /// &str转换为Method
    #[inline]
    pub(super) fn method_from_str(s: &str) -> Option<Method> {
        if let Ok(method) = Method::from_str(s) {
            Some(method)
        } else {
            None
        }
    }
}

impl HTTPMetadata {
    pub fn new(request: &[u8]) -> Option<Self> {
        let request = String::from_utf8_lossy(request);
        let mut lines = request.lines();

        //第一行，包括Method Version Path
        let method_path_version = lines.next()?.split_whitespace().collect::<Vec<_>>();
        let method = functions::method_from_str(method_path_version.get(0)?)?;
        let path = method_path_version.get(1)?.to_string();
        let version = version_from_str(method_path_version.get(2)?)?;

        //kv行，如果有字符串是空字符串则接下来是body行
        let mut kv = HashMap::new();
        loop {
            if let Some(line) = lines.next() {
                //kv行结束了
                if line == "" {
                    break;
                } else {
                    let temp = line.split(':').collect::<Vec<&str>>();
                    if let (Some(k), Some(v)) = (temp.get(0), temp.get(1)) {
                        kv.insert(k.to_string(), v.to_string());
                    }
                }
            } else {
                //kv行结束了，没有后面body的必要，直接return
                return Some(Self {
                    method,
                    version,
                    path,
                    kv,
                    body: None,
                });
            }
        }

        //body数据处理(Cow<str> -> Vec<u8>)
        let mut body = Vec::<u8>::new();
        for i in lines {
            body.extend(i.as_bytes());
            body.extend(b"\r\n");
        }

        Some(Self {
            method,
            version,
            path,
            kv,
            body: Some(body),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let request = String::from("GET /foo/bar HTTP/1.1\r\nHost: test\r\n\r\na");
        let metadata = HTTPMetadata::new(request.as_bytes()).unwrap();
        println!("{:#?}", metadata);
    }
}
