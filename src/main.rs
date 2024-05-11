use std::env;
use std::ops::Deref;

use itertools::Itertools;

use crate::http::Body;
use crate::http::Headers;
use crate::http::request::HTTPMethod;
use crate::http::response::Response;
use crate::server::Server;

mod server;
mod http;

// struct HTTPVersion;
// struct HTTPStatusCode;
// struct HTTPStatusText;
// struct HTTPHeader;
// struct HTTPBody;

// struct HTTPStatusLine {
//     http_version: HTTPVersion,
//     status_code: HTTPStatusCode,
//     status_text: HTTPStatusText,
// }
// struct HTTPResponse {
//     status: HTTPStatusLine,
//     headers: Vec<HTTPHeader>,
//     body:HTTPBody,
// }
fn main() {
    // let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let args: Vec<String> = env::args().collect();
    let found = args.iter().find_position(|s| { s.contains("--directory") });
    let argument_value_index = match found {
        None => None,
        Some((index, _)) => args.get(index + 1)
    };

    let mut server = Server::new("127.0.0.1", 4221, argument_value_index.cloned());

    server.register_handler("/echo".to_string(), Some(false), |request, _| {
        let resource = &request.resource;
        let resource_parts = resource.split_once("/echo/").unwrap().1;
        let body: Body = resource_parts.parse().unwrap();
        let mut headers = Headers::new();
        headers.insert("Content-Type".to_string(), ["text/plain".to_string()].to_vec());
        let accept_encoding_header_value = match request.headers.get("Accept-Encoding") {
            None => None,
            Some(value) => value.get(0)
        };


        match accept_encoding_header_value {
            None => {
                headers.insert("Content-Length".to_string(), [format!("{}", body.len()).to_string()].to_vec());
            }
            Some(header) => {
                if header == "gzip" {
                    headers.insert("Content-Encoding".to_string(), ["gzip".to_string()].to_vec());
                }
            }
        }

        Response::new("HTTP/1.1 200 OK".to_string(), headers, Option::Some(body))

    });

    server.register_handler("/index.html".to_string(), Some(true), |_, _| {
        Response::new("HTTP/1.1 404 Not Found".to_string(), Headers::new(), Option::None)
    });

    server.register_handler("/user-agent".to_string(), Some(true), |r, _| {
        let headers = &r.headers;
        let user_agent = match headers.get("User-Agent") {
            None => None,
            Some(value) => Some(value.get(0).unwrap())
        };

        let response_body: Body = match user_agent {
            None => Body::default(),
            Some(value) => value.parse().unwrap()
        };

        let mut response_headers = Headers::new();
        response_headers.insert("Content-Type".to_string(), ["text/plain".to_string()].to_vec());
        response_headers.insert("Content-Length".to_string(), [format!("{}", response_body.len()).to_string()].to_vec());
        return Response::new("HTTP/1.1 200 OK".to_string(), response_headers, Some(response_body));
    });

    server.register_handler("/".to_string(), Some(true), |_, _| {
        Response::new("HTTP/1.1 200 OK".to_string(), Headers::new(), Option::None)
    });
    server.register_handler("/files".to_string(), Some(false), |req, dir| {
        let not_found = Response::new("HTTP/1.1 404 Not Found".to_string(), Headers::new(), Option::None);
        match req.method {
            HTTPMethod::GET => match dir.deref() {
                None => not_found,
                Some(dir_path) => {
                    println!("{:?}", dir_path);
                    match req.resource.split_once("/files/") {
                        None => not_found,
                        Some((_, file_name)) => {
                            match std::fs::read_to_string(format!("{dir_path}/{file_name}")) {
                                Ok(content) =>
                                    {
                                        let body: Body = content.parse().unwrap();
                                        let mut response_headers = Headers::new();
                                        response_headers.insert("Content-Type".to_string(), ["application/octet-stream".to_string()].to_vec());
                                        response_headers.insert("Content-Length".to_string(), [body.len().to_string()].to_vec());
                                        Response::new("HTTP/1.1 200 OK".to_string(), response_headers, Some(body))
                                    }
                                Err(_) => not_found
                            }
                        }
                    }
                }
            }

            HTTPMethod::POST => match dir.deref() {
                None => not_found,
                Some(dir_path) => {
                    println!("{:?}", dir_path);
                    match req.resource.split_once("/files/") {
                        None => not_found,
                        Some((_, file_name)) => {
                            let body = &req.body;
                            std::fs::write(format!("{dir_path}/{file_name}"), body).unwrap();
                            println!("Req: {:?}", req.body);
                            Response::new("HTTP/1.1 201 Created".to_string(), Headers::new(), None)
                        }
                    }
                }
            }
        }
    });

    server.serve()
}