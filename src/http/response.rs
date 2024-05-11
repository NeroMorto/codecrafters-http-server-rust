use std::io::{BufWriter, Write};
use std::ops::Add;
use crate::http::{Body, Headers};

#[derive(Debug)]
pub struct Response {
    status: String,
    headers: Headers,
    body: Option<Body>,
}

impl Response {
    pub fn new(status: String, headers: Headers, body: Option<Body>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    pub fn try_into_bytes(&self) -> BufWriter<Vec<u8>> {
        let mut buf = BufWriter::new(Vec::with_capacity(1));
        buf.write(self.status.as_bytes()).unwrap();
        buf.write("\r\n".as_bytes()).unwrap();
        let headers = self.headers.clone().iter().map(|(header_name, header_value)| {
            let header_line = header_name;
            let header_value = header_value.join(", ");
            return header_line.clone().add(": ").add(header_value.as_str());
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
