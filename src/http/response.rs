use std::io::{BufWriter, Write};

use crate::http::{Body, Headers};

struct Status(i8, String);

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
    pub headers: Headers,
    pub body: Option<Body>,
    http_version: Option<String>,
}

impl Response {
    // TODO
    // Add set_body, set_headers
    // Add method for response headers separator and body separator
    // Add method to add body length header (or add it by default?)
    pub fn new(status: HTTPStatus) -> Self {
        Self {
            status,
            headers: Headers::new(),
            body: None,
            http_version: None,
        }
    }

    pub fn set_http_version(&mut self, http_version: &str) {
        self.http_version = Some(http_version.to_string())
    }

    pub fn try_into_bytes(&self) -> BufWriter<Vec<u8>> {
        let mut buf = BufWriter::new(Vec::with_capacity(1));

        let status_line = format!(
            "{http_version} {status}",
            status = self.status.to_string(),
            http_version = self.http_version.clone().unwrap_or("HTTP/1.1".to_string())
        );

        buf.write(status_line.as_bytes()).unwrap();

        buf.write("\r\n".as_bytes()).unwrap();

        let headers = self.headers.iter().map(|(header_name, header_value)| {
            return format!("{header_name}: {values}", values = header_value.join(", "));
        }).collect::<Vec<_>>().join("\r\n");

        buf.write(headers.as_bytes()).unwrap();
        buf.write("\r\n".as_bytes()).unwrap();


        match &self.body {
            None => {}
            Some(body) => {
                buf.write("\r\n".as_bytes()).unwrap();
                buf.write(body.content.as_slice()).unwrap();
            }
        }
        buf
    }
}
