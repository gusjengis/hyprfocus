mod daemon_commands;
mod log_parsing;
mod view;

use daemon_commands::send_command;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--idle") => send_command("idle"),
        Some("--resume") => send_command("resume"),
        Some("--class") | Some("-c") => {
            let class_arg = args.get(2).map(String::as_str);
            match class_arg {
                Some(class) => {
                    view::render_log(class);
                }
                None => {
                    println!("Missing class_name argument. Please specify a class.")
                }
            }
            std::process::exit(1);
        }
        Some("--titles") | Some("-t") => {
            view::render_log("*");
            std::process::exit(1);
        }
        Some(arg) => {
            eprintln!("Unknown argument: {arg}");
            print_usage();
            std::process::exit(1);
        }
        None => {
            view::render_log("");
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!(
        "Usage: hyprfocus [--titles | -t | --class CLASS_NAME | -c CLASS_NAME | --idle | --resume]"
    );
}
