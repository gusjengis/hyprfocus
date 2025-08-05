use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::write_to_log;

const SOCKET_PATH: &str = "/tmp/hyprfocus.sock";

pub async fn start_socket_listener() -> std::io::Result<()> {
    // Remove stale socket
    if Path::new(SOCKET_PATH).exists() {
        if fs::metadata(SOCKET_PATH)?.file_type().is_socket() {
            fs::remove_file(SOCKET_PATH)?;
        }
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_client(stream));
    }
}

async fn handle_client(stream: UnixStream) {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        match line.trim() {
            "idle" => write_to_log("SYSTEM", "idle"),
            "resume" => write_to_log("SYSTEM", "resume"),
            _ => eprintln!("Unknown message: {}", line),
        }
    }
}

pub enum SessionState {
    Active,
    Idle {
        buffered: Vec<(String, String)>, // Vec<(class, title)>
    },
}
