use std::io::{BufWriter, Write};

use crate::http::{Body, HeaderName};
use crate::http::headers::{HeaderMap, HTTPHeader};

#[derive(Debug)]
pub enum HTTPStatus {
    Ok,
    Created,
    NotFound,
}

impl HTTPStatus {
    pub fn to_string(&self) -> &'static str {
        match self {
            HTTPStatus::Ok => "200 OK",
            HTTPStatus::Created => "201 Created",
            HTTPStatus::NotFound => "404 Not Found"
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: HTTPStatus,
    pub headers: HeaderMap,
    pub body: Option<Body>,
    http_version: Option<String>,
}

impl Response {
    const LINE_FEED: &'static str = "\r\n";
    // TODO
    // Add set_body, set_headers
    // Add method for response headers separator and body separator
    // Add method to add body length header (or add it by default?)
    pub fn new(status: HTTPStatus) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: None,
            http_version: None,
        }
    }

    pub fn set_http_version(&mut self, http_version: &str) {
        self.http_version = Some(http_version.to_string())
    }

    pub fn set_body(&mut self, body: Body) {
        self.body = Some(body)
    }

    pub fn add_known_header(&mut self, header_name: HTTPHeader, header_values: Vec<&str>) {
        self.insert_header_values(HeaderName::Known(header_name), header_values);
    }
    #[allow(dead_code)]
    pub fn add_custom_header(&mut self, header_name: String, header_values: Vec<&str>) {
        self.insert_header_values(HeaderName::Custom(header_name), header_values);
    }

    fn insert_header_values(&mut self, header_name: HeaderName, header_values: Vec<&str>) {
        let header_name = match header_name {
            HeaderName::Known(header) => header.to_string(),
            HeaderName::Custom(header) => header
        };
        self.headers.insert(header_name, header_values.iter().map(|&value| { value.to_string() }).collect());
    }

    fn write_line_feed(buffer: &mut BufWriter<Vec<u8>>) -> std::io::Result<usize> {
        // Response::LINE_FEED.as_bytes()
        buffer.write(Response::LINE_FEED.as_bytes())
    }

    pub fn set_content_length_header(&mut self) {
        match &self.body {
            None => {}
            Some(body) => self.add_known_header(HTTPHeader::ContentLength, vec![body.len().to_string().as_str()])
        }
    }

    pub fn try_into_bytes(&self) -> BufWriter<Vec<u8>> {
        let mut buf = BufWriter::new(Vec::with_capacity(1));

        let status_line = format!(
            "{http_version} {status}",
            status = self.status.to_string(),
            http_version = self.http_version.clone().unwrap_or("HTTP/1.1".to_string())
        );

        buf.write(status_line.as_bytes()).unwrap();

        let _ = Response::write_line_feed(&mut buf);

        // TODO Move to Headers.try_into_bytes()
        let headers = self.headers.iter().map(|(header_name, header_value)| {
            return format!("{header_name}: {values}", values = header_value.join(", "));
        }).collect::<Vec<_>>().join(Response::LINE_FEED);

        buf.write(headers.as_bytes()).unwrap();
        let _ = Response::write_line_feed(&mut buf);


        match &self.body {
            None => {}
            Some(body) => {
                let _ = Response::write_line_feed(&mut buf);
                buf.write(body.content.as_slice()).unwrap();
            }
        }
        buf
    }
}
