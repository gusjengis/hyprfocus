mod view;

use std::{
    env::{self, home_dir},
    fs::{OpenOptions, create_dir_all, metadata},
    io::Write,
    path::PathBuf,
};

use chrono::Local;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--idle") => {
            write_to_log("SYSTEM", "idle");
        }
        Some("--resume") => {
            write_to_log("SYSTEM", "resume");
        }
        Some(arg) => {
            eprintln!("Unknown argument: {arg}");
            print_usage();
            std::process::exit(1);
        }
        None => {
            view::render_log();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Usage: hyprfocus [--idle | --resume]");
}

fn write_to_log(class: &str, title: &str) {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let mut dir: PathBuf = home_dir().expect("could not get home dir");
    dir.push(".local/share/hyprfocus");
    let path = dir.join(format!("{}.csv", date_str));

    create_dir_all(dir).expect("failed to create data directory");
    let file_exists = metadata(&path).is_ok();

    let timestamp = chrono::Local::now().timestamp_millis();
    let line = format!("{},{},{}\n", timestamp, class, title);

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
