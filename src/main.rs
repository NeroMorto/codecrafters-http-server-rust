use std::fmt::format;
mod server;
pub use crate::server::Server;
// use nom::AsBytes;
use crate::server::{Headers, HTTPMethod, Request, Response};

// use std::io::{BufRead, Read, Write};
use std::net::TcpListener;

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
    let mut server = Server::new("127.0.0.1", 4221);

    server.register_handler("/echo".to_string(), Some(false), |request| {
        let resource = &request.resource;
        println!("RESOURCE: {:?}", resource);
        let resource_parts = resource.split("/echo/").map(|part| part.try_into().unwrap()).collect::<Vec<&str>>();

        let response_body  = *resource_parts.last().unwrap();
        println!("RESOURCE: {:?}", resource_parts);
        let mut headers = Headers::new();
        headers.insert("Content-Type".to_string(), ["text/plain".to_string()].to_vec());
        headers.insert("Content-Length".to_string(), [format!("{}", response_body.len()).to_string()].to_vec());
        return Response::new("HTTP/1.1 200 OK".to_string(), headers, Option::Some(response_body.to_string()));
    });

    server.register_handler("/index.html".to_string(), Some(true),|_| {
       Response::new("HTTP/1.1 404 NOT FOUND".to_string(), Headers::new(), Option::None)
    });

    server.register_handler("/user-agent".to_string(), Some(true), |r| {
        let headers = &r.headers;
        let user_agent = match headers.get("User-Agent") {
            None => None,
            Some(value) => Some(value.get(0).unwrap())
        };
        println!("{:?}", user_agent);
        let response_body = match user_agent {
            None => "",
            Some(value) => value
        };
        // let user_agent = headers.get("User-Agent").unwrap();
        let mut response_headers = Headers::new();
        response_headers.insert("Content-Type".to_string(), ["text/plain".to_string()].to_vec());
        response_headers.insert("Content-Length".to_string(), [format!("{}", response_body.len()).to_string()].to_vec());
        let resp =  Response::new("HTTP/1.1 200 OK".to_string(), response_headers, Some(response_body.to_string()));
        println!("{:?}", resp);
        resp
    });

    server.register_handler("/".to_string(), Some(true),|r| {
        Response::new("HTTP/1.1 200 OK".to_string(), Headers::new(), Option::None)
    });

    server.serve()


    // let ok_200: String = "HTTP/1.1 200 OK\r\n\r\n".to_owned();
    // let not_found_404: String = "HTTP/1.1 404 Not Found\r\n\r\n".to_owned();
    //
    // for stream in listener.incoming() {
    //     match stream {
    //         Ok(mut stream) => {
    //             println!("accepted new connection");
    //
    //             // let reader = BufReader::new(stream);
    //
    //
    //             let mut buffer = [0; 1024];
    //             stream.read(&mut buffer).unwrap();
    //
    //             let request_lines = String::from_utf8_lossy(&mut buffer);
    //             for line in request_lines.split("\r\n").into_iter() {
    //                 match line {
    //                     line => {
    //                         if line.starts_with("GET / HTTP") {
    //                             stream.write(ok_200.as_bytes()).unwrap();
    //                         }
    //                         if line.contains("GET /echo/") {
    //                             let (_, rest) = line.split_once(' ').unwrap();
    //                             let (target, http_version) = rest.split_once(' ').unwrap();
    //
    //                             let target_parts = target.split("/echo/").map(|part| part.try_into().unwrap()).collect::<Vec<&str>>();
    //
    //                             let body = target_parts.last().unwrap();
    //                             let content_length = body.as_bytes().len();
    //
    //                             // TODO Create a Request struct
    //                             stream.write(format!("{http_version} 200 OK\r\n").as_bytes()).unwrap();
    //                             // TODO Rewrite with structs (Response and its elements)
    //                             stream.write("Content-Type: text/plain".as_bytes()).unwrap();
    //                             stream.write("\r\n".as_bytes()).unwrap();
    //                             stream.write(format!("Content-Length: {content_length}").as_bytes()).unwrap();
    //                             stream.write("\r\n\r\n".as_bytes()).unwrap();
    //                             stream.write(body.as_bytes()).unwrap();
    //                             break;
    //                         }
    //
    //                         stream.write(not_found_404.as_bytes()).unwrap();
    //                     }
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             println!("error: {}", e);
    //         }
    //     }
    // }
}