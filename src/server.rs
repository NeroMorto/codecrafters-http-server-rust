use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::Config;
use crate::http::{Headers, response};
use crate::http::request::Request;
use crate::http::response::{HTTPStatus, Response};
use crate::route::Router;

type RequestHandler = fn(request: &Request, config: &Config) -> Response;
type ExactResource = bool;

pub struct Server {
    handlers: Arc<Mutex<HashMap<String, (ExactResource, RequestHandler)>>>,
    config: Arc<Config>,
    pub router: Arc<Router>,
}

impl Server {
    pub fn new(config: Config, router: Router) -> Server {
        let handlers = Arc::new(Mutex::new(HashMap::with_capacity(1)));
        Self {
            handlers,
            config: Arc::new(config),
            router: Arc::new(router),
        }
    }

    fn handle_request(request: &Request, router: &Arc<Router>, config: &Arc<Config>) -> Response {
        let possible_handler: Option<&RequestHandler> = router.routes.iter().find_map(|route| {

            // Rewrite with regular expression
            if request.resource == '/'.to_string() {
                if request.resource == route.path && request.method == route.method {
                    println!("EXACT FOUND: {:?}, {:?}", route.path, route.method);
                    return Some(&route.handler);
                }
            } else {
                if request.resource.starts_with(route.path.as_str()) && route.path != '/'.to_string() && request.method == route.method {
                    println!("starts_with FOUND: {:?}, {:?}", route.path, route.method);
                    return Some(&route.handler);
                }
            }

            println!("REQUEST: {request}");
            println!("NOT FOUND: {:?}, {:?}", route.path, route.method);

            None
        });

        let handler: RequestHandler = match possible_handler {
            None => RequestHandler::from(|_, _| { Response::new(HTTPStatus::NotFound) }),
            Some(handler) => *handler
        };
        let mut response = handler(&request, &config);
        response.set_http_version(&request.http_version);
        response
    }


    pub fn serve(&self) {
        let address = format!("{hostname}:{port}", hostname = self.config.address, port = self.config.port);
        let listener: TcpListener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming().flatten() {

            // TODO try to move inside request_handler
            let reader = BufReader::new(&stream);
            let request = Request::new(reader).unwrap();
            let config = self.config.clone();
            let router = self.router.clone();
            thread::spawn(move || {
                // println!("Request: {:?}", reader);
                let mut response = Server::handle_request(&request, &router, &config);
                
                let mut writer = BufWriter::new(&stream);
                // let response = handler(&request, &config);
                writer.write(response.try_into_bytes().buffer())
            });
        }
    }
}
