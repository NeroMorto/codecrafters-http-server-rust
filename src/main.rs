use std::io::Write;
// Uncomment this block to pass the first stage
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let ok_200: String = "HTTP/1.1 200 OK\r\n\r\n".to_owned();
                let response = stream.write(ok_200.as_bytes());
                match response {
                    Ok(_) => { println!("Responded successfully"); }
                    Err(error) => {println!("Error during response: {:?}", error);}
                }

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
