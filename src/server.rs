use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::ops::Add;

use itertools::Itertools;
use nom::AsBytes;

#[derive(Debug)]
pub enum HTTPMethod {
    GET,
    POST,
}

type RequestHandler = fn(request: &Request) -> Response;
type ExactResource = bool;

pub struct Server {
    connection: TcpListener,
    handlers: HashMap<String, (ExactResource, RequestHandler)>,
}

struct RequestLine {
    http_method: HTTPMethod,
    resource: String,
    http_version: String,
}

pub type Headers = std::collections::HashMap<String, Vec<String>>;
type Body = Vec<u8>;

#[derive(Debug)]
pub struct Request {
    pub resource: String,
    pub method: HTTPMethod,
    pub headers: Headers,
    pub body: Body,
}


impl Request {
    pub fn new(mut stream: BufReader<&TcpStream>) -> io::Result<Self> {
        // let mut request_buffer = Vec::with_capacity(0x1000);

        let request_line = Request::read_request_line(&mut stream)?;
        let headers = Request::read_headers(&mut stream)?;
        // let content_length_value = headers.get("Content-Length").unwrap();
        // let content_length = content_length_value.get(0).unwrap().parse().unwrap();
        let content_length = match headers.get("Content-Length") {
            None => 0,
            Some(value) => value.get(0).unwrap().parse().unwrap()
        };
        let body = Request::read_body(&mut stream, content_length)?;


        Ok(Self {
            resource: request_line.resource,
            method: request_line.http_method,
            headers,
            body,
        })
    }

    fn read_headers(stream: &mut BufReader<&TcpStream>) -> io::Result<Headers> {
        let mut headers: Headers = Headers::new();

        loop {
            let header_line = Request::read_header_line(stream)?;
            if header_line.is_empty() {
                break;
            }

            let mut header_name_and_value = header_line.split(": ");
            let name = header_name_and_value.next().unwrap().to_string();
            let value = header_name_and_value.next().unwrap().to_string();

            let header_value_slot = headers.entry(name).or_insert(Vec::with_capacity(1));
            header_value_slot.push(value);
        }

        Ok(headers)
    }


    fn read_request_line(stream: &mut BufReader<&TcpStream>) -> io::Result<RequestLine> {
        let request_line = Request::read_header_line(stream).unwrap();
        let mut parts = request_line.split_ascii_whitespace();
        let http_method = match parts.next().unwrap() {
            "GET" => HTTPMethod::GET,
            "POST" => HTTPMethod::POST,
            _ => return Err(io::Error::new(ErrorKind::InvalidData, "Unsupported HTTP method"))
        };
        let resource = parts.next().unwrap().to_string();
        let http_version = parts.next().unwrap().to_string();

        Ok(RequestLine {
            http_method,
            resource,
            http_version,
        })
    }
    fn read_header_line(stream: &mut BufReader<&TcpStream>) -> io::Result<String> {
        let mut buf: Vec<u8> = Vec::with_capacity(0x1000);
        while let Some(Ok(byte)) = stream.bytes().next() {
            if byte == b'\n' {
                if buf.ends_with(b"\r") {
                    buf.pop();
                }
                let header_line = String::from_utf8(buf).map_err(|_| { io::Error::new(ErrorKind::InvalidData, "Not a HTTP header") })?;
                return Ok(header_line);
            }
            buf.push(byte);
        }

        Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Client aborted early"))
    }

    fn read_body(stream: &mut BufReader<&TcpStream>, content_length: usize) -> io::Result<Body> {
        if content_length == 0 {
            return Ok(Vec::new());
        }
        // let mut str_bod = String::new();
        let mut body = Vec::with_capacity(content_length);
        // println!("BODY: {:?}", stream.bytes().size_hint());
        let _ = stream.read_exact(&mut body)?;
        // println!("BODY: {:?}", str_bod);
        Ok(body)
    }
}

#[derive(Debug)]
pub struct Response {
    status: String,
    headers: Headers,
    body: Option<String>,
}

impl Response {
    pub fn new(status: String, headers: Headers, body: Option<String>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    pub fn try_into_bytes(&self) -> BufWriter<Vec<u8>> {
        let mut buf = BufWriter::new(Vec::with_capacity(1));
        buf.write(self.status.as_bytes());
        buf.write("\r\n".as_bytes());
        let headers = self.headers.clone().iter().map(|(header_name, header_value)| {
            let mut header_line = header_name;
            let header_value = header_value.join(", ");
            return header_line.clone().add(": ").add(header_value.as_str());
        }).join("\r\n");

        buf.write(headers.as_bytes());
        buf.write("\r\n".as_bytes());


        match &self.body {
            None => {}
            Some(content) => {
                // println!("Content: {:?}", content);
                // buf.write("\r\n".as_bytes());
                buf.write("\r\n".as_bytes());
                buf.write(content.as_bytes());
            }
        }

        // buf.write(body.as_bytes());
        buf
    }
}

impl Server {
    pub fn new(address: &str, port: i32) -> Self {
        let listener: TcpListener = TcpListener::bind(format!("{address}:{port}")).unwrap();
        let mut handlers = HashMap::with_capacity(1);
        Self {
            connection: listener,
            handlers,
        }
    }

    pub fn register_handler(&mut self, resource: String, exact_resource: Option<ExactResource>, handler: RequestHandler) {
        let exact_resource = match exact_resource {
            None => false,
            Some(v) => v
        };
        self.handlers.insert(resource, (exact_resource, handler));
    }

    pub fn serve(&self) {
        for stream in self.connection.incoming().flatten() {
            let reader = BufReader::new(&stream);
            let request = Request::new(reader).unwrap();

            println!("Request: {:?}", request);
            let mut writer = BufWriter::new(&stream);
            let possible_handler = &self.handlers.iter().find(|(res, (exact, handler))| {
                match exact {
                    &true => {
                        if request.resource == res.as_str() {
                            return true
                        }
                        false
                    },
                    &false => {
                        if request.resource.contains(res.as_str()) {
                            return true
                        }
                        false
                    }
                }
            });
            let handler_404 = RequestHandler::from(|_: &Request| { Response::new("HTTP/1.1 404 Not found".to_string(), Headers::new(), None) });
            let handler: &RequestHandler = match possible_handler {
                None => &handler_404,
                Some((_res, (_exact, handler))) => handler
            };

            let response = handler(&request);
            writer.write(response.try_into_bytes().buffer());
        }
    }
}
