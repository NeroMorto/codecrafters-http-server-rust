use std::io::{BufRead, Read, Write};
use std::net::TcpListener;

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
                                break;
                            }
                            stream.write(not_found_404.as_bytes()).unwrap();
                        }
                    }
                }


                // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

                // let ok_200: String = "HTTP/1.1 200 OK\r\n\r\n".to_owned();
                // stream.write(ok_200.as_bytes()).unwrap();
                // match response {
                //     Ok(_) => { println!("Responded successfully"); }
                //     Err(error) => { println!("Error during response: {:?}", error); }
                // }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
