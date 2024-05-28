use crate::http::{Body, Headers, response};
use crate::http::request::HTTPMethod;
use crate::http::response::{HTTPStatus, Response};
use crate::route::Route;

pub fn get_routes() -> Vec<Route> {
    let echo = Route::new(HTTPMethod::GET, "/echo", |request, _| {
        let resource_parts = request.resource.split_once("/echo/").unwrap().1;
        let mut response = Response::new(HTTPStatus::Ok);
        response.body = Some(resource_parts.parse().unwrap());
        response.headers.insert("Content-Type".to_string(), ["text/plain".to_string()].to_vec());
        let accept_encoding_header_value = match request.headers.get("Accept-Encoding") {
            None => None,
            Some(value) => value.get(0)
        };


        match accept_encoding_header_value {
            None => {
                response.headers.insert("Content-Length".to_string(), [format!("{}", response.body.as_ref().expect("Missing body").len()).to_string()].to_vec());
            }
            Some(header) => {
                if header == "gzip" {
                    response.headers.insert("Content-Encoding".to_string(), ["gzip".to_string()].to_vec());
                }
            }
        }

        response
    });

    let root_route = Route::new(HTTPMethod::GET, "/", |request, config| {
        Response::new(HTTPStatus::Ok)
    });
    let index_route = Route::new(HTTPMethod::GET, "/index.html", |request, config| {
        Response::new(HTTPStatus::NotFound)
    });
    let user_agent_route = Route::new(HTTPMethod::GET, "/user-agent", |request, _| {
        let user_agent = match &request.headers.get("User-Agent") {
            None => None,
            Some(value) => Some(value.get(0).unwrap())
        };

        let response_body: Body = match user_agent {
            None => Body::default(),
            Some(value) => value.parse().unwrap()
        };
        let mut response = Response::new(HTTPStatus::Ok);

        response.headers.insert("Content-Type".to_string(), ["text/plain".to_string()].to_vec());
        response.headers.insert("Content-Length".to_string(), [format!("{}", response_body.len()).to_string()].to_vec());

        response.body = Some(response_body);
        response
    });

    let read_files_route = Route::new(HTTPMethod::GET, "/files", |request, config| {
        let not_found = Response::new(HTTPStatus::NotFound);
        match &config.files_path {
            None => not_found,
            Some(dir_path) => {
                println!("{:?}", dir_path);
                match request.resource.split_once("/files/") {
                    None => not_found,
                    Some((_, file_name)) => {
                        match std::fs::read_to_string(format!("{dir_path}/{file_name}")) {
                            Ok(content) =>
                                {
                                    let mut response = Response::new(HTTPStatus::Ok);
                                    let body: Body = content.parse().unwrap();

                                    response.headers.insert("Content-Type".to_string(), ["application/octet-stream".to_string()].to_vec());
                                    response.headers.insert("Content-Length".to_string(), [body.len().to_string()].to_vec());
                                    response.body = Some(body);
                                    response
                                }
                            Err(_) => not_found
                        }
                    }
                }
            }
        }
    });
    let write_files_route = Route::new(HTTPMethod::POST, "/files", |request, config| {
        let not_found = Response::new(HTTPStatus::NotFound);
        // return configuration error?
        match &config.files_path {
            None => not_found,
            Some(dir_path) => {
                println!("{:?}", dir_path);
                match request.resource.split_once("/files/") {
                    None => not_found,
                    Some((_, file_name)) => {
                        let body = &request.body;
                        std::fs::write(format!("{dir_path}/{file_name}"), body).unwrap();

                        println!("Req: {:?}", request.body);

                        Response::new(HTTPStatus::Created)
                    }
                }
            }
        }
    });
    vec![echo, user_agent_route, read_files_route, write_files_route, index_route, root_route]
}