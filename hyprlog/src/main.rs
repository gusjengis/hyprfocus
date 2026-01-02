mod daemon_commands;
mod interval;
mod log_parsing;
mod log_reader;
mod model;
mod settings;
mod tui;
mod view;

use color_eyre::Result;
use daemon_commands::send_command;
use std::env;

use crate::{interval::Interval, tui::App};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--idle") => {
            send_command("idle");
            return Ok(());
        }
        Some("--resume") => {
            send_command("idle");
            return Ok(());
        }
        _ => {
            color_eyre::install()?;
            ratatui::run(|terminal| App::new().run(terminal));
            Ok(())
        }
    }
}
