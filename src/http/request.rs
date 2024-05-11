use std::io::{BufReader, ErrorKind, Read};
use std::io;
use std::net::TcpStream;
use std::str::FromStr;

use crate::http::{Body, Headers, RequestLine};

#[allow(dead_code)]
struct RequestTarget(String);
#[allow(dead_code)]
struct HTTPVersion;
#[allow(dead_code)]
enum HTTPHeader {
    Host,
    UserAgent,
    Accept,
}

#[derive(Debug)]
pub enum HTTPMethod {
    GET,
    POST,
}

#[allow(dead_code)]
pub enum HTTPRequestParseError {
    InvalidMethodError,
    InvalidStatusLineError,
    InvalidPathError
}
impl FromStr for HTTPMethod {
    type Err = HTTPRequestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            _ => Err(HTTPRequestParseError::InvalidMethodError)
        }
    }
}


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
            return Ok(Body::default());
        }
        // let mut str_bod = String::new();
        let mut content: Vec<u8> = vec![0; content_length];
        _ = stream.read_exact(&mut content)?;
        // let body_content = match stream.read_exact(&mut body) {
        //     Ok(content) => println!("Content? :{:?}", content),
        //     Err(err) => println!("Error: {:?}", err)
        // };
        // println!("BODY: {:?}", String::from_utf8_lossy(&body));
        Ok(Body::new(content))
    }
}
