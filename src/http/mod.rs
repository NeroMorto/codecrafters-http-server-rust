use std::str::FromStr;
use crate::http::headers::HTTPHeader;
use crate::http::request::HTTPMethod;

pub mod request;
pub mod response;
pub mod headers;
pub mod compression;

struct RequestLine {
    http_method: HTTPMethod,
    resource: String,
    #[allow(dead_code)]
    http_version: String,
}


// pub type Body = Vec<u8>;
#[derive(Debug)]
pub struct BodyTransformError;

#[derive(Debug, Default, Clone)]
pub struct Body {
    content: Vec<u8>,
}


impl FromStr for Body {
    type Err = BodyTransformError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Body { content: s.bytes().collect() })
    }
}

impl AsRef<[u8]> for Body {
    fn as_ref(&self) -> &[u8] {
        self.content.as_slice()
    }
}

impl Body {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            content: bytes
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}

enum HeaderName {
    Known(HTTPHeader),
    Custom(String),
}