use std::collections::HashMap;
use crate::config::Config;
use crate::http::request::{HTTPMethod, Request};
use crate::http::response::Response;
type RequestHandler = fn(request: &Request, config: &Config) -> Response;
#[derive(Debug)]
pub struct Route {
    pub method: HTTPMethod,
    pub path: String,
    pub handler: RequestHandler
}

impl Route {
    pub fn new(method: HTTPMethod, path: &str, handler: RequestHandler) -> Self {
        Self{
            method,
            path: path.to_string(),
            handler
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct RouterKey(pub String, pub HTTPMethod);
#[derive(Debug)]
pub struct Router {
    pub routes: Vec<Route>
}
impl Router {
    pub fn new (routes: Option<Vec<Route>>) -> Self{
        match routes {
            None => Self{
                routes: Vec::with_capacity(1)
            },
            Some(routes) => {
                Self {
                    routes
                }
            }
        }

    }

    pub fn add_route(&mut self, path: &str, method: HTTPMethod, handler: RequestHandler) {
        self.routes.push(Route{path: path.to_string(), method, handler});
    }
}