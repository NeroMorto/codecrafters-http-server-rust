use std::fmt::Display;

#[derive(Hash, Eq, PartialEq)]
pub enum HTTPHeader {
    UserAgent,
    AcceptEncoding,
    ContentType,
    ContentLength,
    ContentEncoding,
}

impl Display for HTTPHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header_string = match self {
            HTTPHeader::UserAgent => "User-Agent".to_string(),
            HTTPHeader::AcceptEncoding => "Accept-Encoding".to_string(),
            HTTPHeader::ContentType => "Content-Type".to_string(),
            HTTPHeader::ContentLength => "Content-Length".to_string(),
            HTTPHeader::ContentEncoding => "Content-Encoding".to_string()
        };
        write!(f, "{}", header_string)
    }
}

pub type HeaderMap = std::collections::HashMap<String, Vec<String>>;
