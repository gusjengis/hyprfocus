use std::{fs::remove_file, path::Path};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc::Sender,
};

use crate::log_writer::LogMsg;

pub async fn start_socket_listener(sender_handle: Sender<LogMsg>) -> std::io::Result<()> {
    const SOCKET_PATH: &str = "/tmp/hyprfocus.sock";

    if Path::new(SOCKET_PATH).exists() {
        let _ = remove_file(SOCKET_PATH);
    }
    let listener = tokio::net::UnixListener::bind(SOCKET_PATH)?;

    loop {
        let (stream, _) = listener.accept().await?;
        let sender_handle_clone = sender_handle.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stream);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let ts = chrono::Local::now().timestamp_millis();
                match line.trim() {
                    "idle" => {
                        let _ = sender_handle_clone
                            .send(LogMsg::Line {
                                ts,
                                class: "SYSTEM".into(),
                                title: "idle".into(),
                            })
                            .await;
                    }
                    "resume" => {
                        let _ = sender_handle_clone
                            .send(LogMsg::Line {
                                ts,
                                class: "SYSTEM".into(),
                                title: "resume".into(),
                            })
                            .await;
                    }
                    other => eprintln!("Unknown message: {other}"), // output to file
                }
            }
        });
    }
}
