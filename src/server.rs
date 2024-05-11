use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;


use crate::http::Headers;
use crate::http::request::Request;
use crate::http::response::Response;

type RequestHandler = fn(request: &Request, directory_path: Arc<Option<String>>) -> Response;
type ExactResource = bool;

pub struct Server {
    connection: TcpListener,
    handlers: Arc<Mutex<HashMap<String, (ExactResource, RequestHandler)>>>,
    directory_path: Arc<Option<String>>
}

impl Server {
    pub fn new(address: &str, port: i32, directory_path: Option<String>) -> Server {
        let listener: TcpListener = TcpListener::bind(format!("{address}:{port}")).unwrap();
        let handlers = Arc::new(Mutex::new(HashMap::with_capacity(1)));
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
        let possible_handler =handlers.iter().find(|(res, (exact, _handler))| {
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
