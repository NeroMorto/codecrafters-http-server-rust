pub struct Config {
    pub files_path: Option<String>,
    pub address: String,
    pub port: i32,
}

impl<'a> Config {
    pub fn new(address: &str, port: i32, files_path: Option<String>) -> Self {
        Self {
            address: address.to_string(),
            port,
            files_path,
        }
    }
}