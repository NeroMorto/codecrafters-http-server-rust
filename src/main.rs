use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use nom::AsBytes;

struct HTTPVersion;
struct HTTPStatusCode;
struct HTTPStatusText;
struct HTTPHeader;
struct HTTPBody;

struct HTTPStatusLine {
    http_version: HTTPVersion,
    status_code: HTTPStatusCode,
    status_text: HTTPStatusText,
}
struct HTTPResponse {
    status: HTTPStatusLine,
    headers: Vec<HTTPHeader>,
    body:HTTPBody,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let ok_200: String = "HTTP/1.1 200 OK\r\n\r\n".to_owned();
    let not_found_404: String = "HTTP/1.1 404 Not Found\r\n\r\n".to_owned();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                // let reader = BufReader::new(stream);


                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();

                let request_lines = String::from_utf8_lossy(&mut buffer);
                for line in request_lines.split("\r\n").into_iter() {
                    match line {
                        line => {
                            if line.starts_with("GET / HTTP") {
                                stream.write(ok_200.as_bytes()).unwrap();
                            }
                            if line.contains("GET /echo/") {
                                let (_, rest) = line.split_once(' ').unwrap();
                                let (target, http_version) = rest.split_once(' ').unwrap();

                                let target_parts = target.split("/echo/").map(|part| part.try_into().unwrap()).collect::<Vec<&str>>();

                                let body = target_parts.last().unwrap();
                                let content_length = body.as_bytes().len();

                                // TODO Create a Request struct
                                stream.write(format!("{http_version} 200 OK\r\n").as_bytes()).unwrap();
                                // TODO Rewrite with structs (Response and its elements)
                                stream.write("Content-Type: text/plain".as_bytes()).unwrap();
                                stream.write("\r\n".as_bytes()).unwrap();
                                stream.write(format!("Content-Length: {content_length}").as_bytes()).unwrap();
                                stream.write("\r\n\r\n".as_bytes()).unwrap();
                                stream.write(body.as_bytes()).unwrap();
                                break;
                            }

                            stream.write(not_found_404.as_bytes()).unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
