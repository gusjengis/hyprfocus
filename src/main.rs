use std::{
    env::home_dir,
    fs::{OpenOptions, create_dir_all, metadata},
    io::Write,
    path::PathBuf,
};

use chrono::Local;
use hyprland::{
    async_closure,
    event_listener::{AsyncEventListener, WindowEventData},
};

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    let mut event_listener = AsyncEventListener::new();

    event_listener.add_active_window_changed_handler(async_closure! { move |window_data| {
            if let Some(data) = window_data {
            write_window_data(data);
            }
        }
    });

    event_listener.start_listener_async().await
}

fn write_window_data(data: WindowEventData) {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let mut dir: PathBuf = home_dir().expect("could not get home dir");
    dir.push(".local/share/hyprfocus");
    let path = dir.join(format!("{}.csv", date_str));
    create_dir_all(dir).expect("failed to create data directory");
    let file_exists = metadata(&path).is_ok();
    let timestamp = chrono::Local::now().timestamp_millis();
    let line = format!("{},{},{}\n", timestamp, data.class, data.title);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("failed to open log file");

    if !file_exists {
        writeln!(file, "timestamp,class,title").expect("failed to write header");
    }

    file.write_all(line.as_bytes())
        .expect("failed to write to file");
}
