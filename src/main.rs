use std::env;

use itertools::Itertools;

use crate::config::Config;
use crate::route::Router;
use crate::routes::get_routes;
use crate::server::Server;

mod server;
mod http;
mod route;
mod config;
mod routes;

fn main() {
    let args: Vec<String> = env::args().collect();
    let found = args.iter().find_position(|s| { s.contains("--directory") });
    let files_directory_path_string = match found {
        None => None,
        Some((index, _)) => args.get(index + 1).cloned()
    };

    let config = Config::new("127.0.0.1", 4221, files_directory_path_string);
    let router = Router::new(Some(get_routes()));
    let server = Server::new(config, router);

    server.serve()
}
