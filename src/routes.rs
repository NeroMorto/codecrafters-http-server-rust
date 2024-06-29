use crate::http::Body;
use crate::http::compression;
use crate::http::headers::HTTPHeader;
use crate::http::request::HTTPMethod;
use crate::http::response::{HTTPStatus, Response};
use crate::route::Route;

pub fn get_routes() -> Vec<Route> {
    let echo = Route::new(HTTPMethod::GET, "/echo", |request, _| {
        let echo_data = request.resource.split_once("/echo/").unwrap().1;

        let mut response = Response::new(HTTPStatus::Ok);
        response.add_known_header(HTTPHeader::ContentType, vec!["text/plain"]);

        match request.get_known_header_values(HTTPHeader::AcceptEncoding) {
            None => {
                response.set_body(echo_data.parse().unwrap());
                response.set_content_length_header();
            }
            Some(values) => {
                match values.len() {
                    0 => {}
                    _ => {
                        for encoding in values.iter() {
                            if encoding == "gzip" {
                                // TODO Move compression to common place for handling requests
                                // As well as setting content length?
                                match compression::gzip(echo_data) {
                                    Err(_) => {}
                                    Ok(data) => {
                                        response.add_known_header(HTTPHeader::ContentEncoding, vec![encoding]);
                                        response.set_body(Body::new(data));
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        };
        response.set_content_length_header();
        response
    });

    let root_route = Route::new(HTTPMethod::GET, "/", |_request, _config| {
        Response::new(HTTPStatus::Ok)
    });
    let index_route = Route::new(HTTPMethod::GET, "/index.html", |_request, _config| {
        Response::new(HTTPStatus::NotFound)
    });

    let user_agent_route = Route::new(HTTPMethod::GET, "/user-agent", |request, _| {
        let mut response = Response::new(HTTPStatus::Ok);
        let user_agent_values = request.get_known_header_values(HTTPHeader::UserAgent);
        match user_agent_values {
            None => {}
            Some(values) => {
                match values.first() {
                    None => response.set_body(Body::default()),
                    Some(value) => response.set_body(value.parse().unwrap())
                }
            }
        };

        response.set_content_length_header();
        response.add_known_header(HTTPHeader::ContentType, vec!["text/plain"]);
        response
    });

    let read_files_route = Route::new(HTTPMethod::GET, "/files", |request, config| {
        let not_found = Response::new(HTTPStatus::NotFound);
        match &config.files_path {
            None => not_found,
            Some(dir_path) => {
                match request.resource.split_once("/files/") {
                    None => not_found,
                    Some((_, file_name)) => {
                        match std::fs::read_to_string(format!("{dir_path}/{file_name}")) {
                            Ok(content) => {
                                let mut response = Response::new(HTTPStatus::Ok);
                                let body: Body = content.parse().unwrap();
                                response.add_known_header(HTTPHeader::ContentType, vec!["application/octet-stream"]);
                                response.set_body(body);
                                response.set_content_length_header();
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
        match &config.files_path {
            None => not_found,
            Some(dir_path) => {
                match request.resource.split_once("/files/") {
                    None => not_found,
                    Some((_, file_name)) => {
                        let body = &request.body;
                        std::fs::write(format!("{dir_path}/{file_name}"), body).unwrap();
                        Response::new(HTTPStatus::Created)
                    }
                }
            }
        }
    });
    vec![echo, user_agent_route, read_files_route, write_files_route, index_route, root_route]
}