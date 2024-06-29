use std::io::Write;
use std::process::{Command, Stdio};

pub fn gzip(data: &str) -> Result<Vec<u8>, String> {
    let mut gzip = Command::new("gzip")
        .arg("-c")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to create `gzip` command");

    if let Some(gzip_stdin) = gzip.stdin.as_mut() {
        gzip_stdin.write_all(data.as_bytes()).expect("Failed to pass data to compressor");
    }

    let gzip_output = gzip.wait_with_output().expect("Failed to compress data");
    if !gzip_output.status.success() {
        return Err("Unable to gzip data!".to_string());
    }

    Ok(gzip_output.stdout)
}
