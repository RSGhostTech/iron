pub mod client;
pub mod metadata;
mod router;
pub mod service;


pub(super) mod functions {
    use http::Version;

    /// 由于http crate没有做&str转换Version的方式，故实现
    #[inline]
    pub(super) fn version_from_str(s: &str) -> Option<Version> {
        match s {
            "HTTP/0.9" => Some(Version::HTTP_09),
            "HTTP/1.0" => Some(Version::HTTP_10),
            "HTTP/1.1" => Some(Version::HTTP_11),
            _ => None,
        }
    }

    /// 由于http crate没有做Version转换String的方式，故实现
    #[inline]
    pub(super) fn version_into_string(version: Version) -> String {
        match version {
            Version::HTTP_09 => String::from("HTTP/0.9"),
            Version::HTTP_10 => String::from("HTTP/1.0"),
            Version::HTTP_11 => String::from("HTTP/1.1"),
            Version::HTTP_2 => String::from("HTTP/2"),
            Version::HTTP_3 => String::from("HTTP/3"),
            _ => String::from("HTTP/1.1"),
        }
    }
}