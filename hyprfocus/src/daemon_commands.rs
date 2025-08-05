use std::io::Write;
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/hyprfocus.sock";

pub fn send_command(command: &str) {
    match UnixStream::connect(SOCKET_PATH) {
        Ok(mut stream) => {
            writeln!(stream, "{}", command).unwrap_or_else(|e| {
                eprintln!("Failed to write to socket: {}", e);
            });
        }
        Err(e) => {
            eprintln!("Failed to connect to socket: {}", e);
        }
    }
}
