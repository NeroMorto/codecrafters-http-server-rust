use std::{io, thread};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::{Add, Deref};
use std::sync::{Arc, Mutex};

use itertools::Itertools;
use nom::AsBytes;

#[derive(Debug)]
pub enum HTTPMethod {
    GET,
    POST,
}

type RequestHandler = fn(request: &Request, directory_path: Arc<Option<String>>) -> Response;
type ExactResource = bool;

pub struct Server {
    connection: TcpListener,
    handlers: Arc<Mutex<HashMap<String, (ExactResource, RequestHandler)>>>,
    directory_path: Arc<Option<String>>
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
        let mut body: Vec<u8> = vec![0; content_length];
        _ = stream.read_exact(&mut body)?;
        // let body_content = match stream.read_exact(&mut body) {
        //     Ok(content) => println!("Content? :{:?}", content),
        //     Err(err) => println!("Error: {:?}", err)
        // };
        // println!("BODY: {:?}", String::from_utf8_lossy(&body));
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
    pub fn new(address: &str, port: i32, directory_path: Option<String>) -> Server {
        let listener: TcpListener = TcpListener::bind(format!("{address}:{port}")).unwrap();
        let mut handlers = Arc::new(Mutex::new(HashMap::with_capacity(1)));
        let directory_path= Arc::new(directory_path);
        Self {
            connection: listener,
            handlers,
            directory_path
        }
    }

    pub fn register_handler(&mut self, resource: String, exact_resource: Option<ExactResource>, handler: RequestHandler) {
        let exact_resource = match exact_resource {
            None => false,
            Some(v) => v
        };
        let mut handlers = self.handlers.deref().lock().unwrap();
        handlers.insert(resource, (exact_resource, handler));
        // self.handlers.insert(resource, (exact_resource, handler));
    }

    // outer function used instead. Have to find a way to work with threads and self
    fn resolve_handler(&self, resource: &String) -> RequestHandler {
        let handlers = self.handlers.deref().lock().unwrap();
        let possible_handler =handlers.iter().find(|(res, (exact, handler))| {
            match exact {
                &true => {
                    if resource == res.as_str() {
                        return true
                    }
                    false
                },
                &false => {
                    if resource.contains(res.as_str()) {
                        return true
                    }
                    false
                }
            }
        });
        let handler_404 = RequestHandler::from(|_, _| { Response::new("HTTP/1.1 404 Not Found".to_string(), Headers::new(), None) });
        let handler: &RequestHandler = match possible_handler {
            None => &handler_404,
            Some((_res, (_exact, handler))) => handler
        };
        *handler
    }

    pub fn serve(&self) {
        for stream in self.connection.incoming().flatten() {
            // TODO Yes this is bad, I have to find a way to solve this issue somehow
            let handlers = self.handlers.clone();
            let files_dir = self.directory_path.clone();
            match files_dir.deref() {
                None => {}
                Some(str) => {
                    println!("is str?{:?}", str)
                }
            }
            let reader = BufReader::new(&stream);
            let request = Request::new(reader).unwrap();
            let handler = self.resolve_handler(&request.resource);
            thread::spawn(move ||{
                println!("Request: {:?}", request);

                let mut writer = BufWriter::new(&stream);
                let response = handler(&request, files_dir);
                writer.write(response.try_into_bytes().buffer())
            });
        }
    }
}

fn resolve_handler(resource: &String, handlers: &Arc<Mutex<HashMap<String, (ExactResource, RequestHandler)>>>) -> RequestHandler {
    let handlers = handlers.deref().lock().unwrap();
    let possible_handler = handlers.iter().find(|(res, (exact, handler))| {
        match exact {
            &true => {
                if resource == res.as_str() {
                    return true
                }
                false
            },
            &false => {
                if resource.contains(res.as_str()) {
                    return true
                }
                false
            }
        }
    });
    let handler_404 = RequestHandler::from(|_, _| { Response::new("HTTP/1.1 404 Not Found".to_string(), Headers::new(), None) });
    let handler: &RequestHandler = match possible_handler {
        None => &handler_404,
        Some((_res, (_exact, handler))) => handler
    };
    *handler
}