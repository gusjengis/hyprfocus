mod daemon_commands;
mod log_parsing;
mod log_reader;
mod view;

use chrono::{TimeDelta, Utc};
use daemon_commands::send_command;
use std::{collections::HashMap, env};
use view::render_log;

fn main() {
    // use chrono::Utc;
    // let start = Utc::now().timestamp_millis();
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--idle") => send_command("idle"),
        Some("--resume") => send_command("resume"),
        Some("--help") | Some("-h") => {
            print_usage();
        }
        None => {
            view::render_log(&Settings::new());
        }
        _ => {
            let mut settings = Settings::new();
            // Are we waiting on values for these args?
            let mut class = false;
            let mut days = false;
            for arg in args.iter().skip(1) {
                if class {
                    settings.class_arg = match settings.class_mappings.get(arg) {
                        Some(filtered_class) => filtered_class.clone(),
                        None => arg.clone(),
                    };
                    class = false;
                } else if days {
                    match arg.clone().parse::<u64>() {
                        Ok(day_count) => {
                            settings.interval.set_days(day_count);
                            days = false;
                        }
                        Err(_) => {
                            println!("Invalid value for the days argument.");
                            return;
                        }
                    }
                } else {
                    match arg.as_str() {
                        "--class" | "-c" => {
                            class = true;
                        }
                        "--days" | "-d" => {
                            days = true;
                        }
                        "--full" | "-f" => {
                            settings.full = true;
                        }
                        "--multi" | "-m" => {
                            settings.multi_timeline = true;
                        }

                        arg => {
                            eprintln!("Unknown argument: {arg}");
                            print_usage();
                            std::process::exit(1);
                        }
                    }
                }
            }

            if class {
                println!("Please provied a class name for the class argument.");
                return;
            }
            if days {
                println!("Please provied a day count for the days argument.");
                return;
            }

            render_log(&settings);
        }
    }

    // let end = Utc::now().timestamp_millis();
    // println!("Runtime: {}ms", end - start)
}

fn print_usage() {
    println!(
        "Usage: hyprlog\n
        [ --help | -h ]\n
        [ --full | -f ]\n
        [ --multi | -m ]\n
        [ --days DAY_COUNT | -d DAY_COUNT ]\n
        [ --class CLASS_NAME | -c CLASS_NAME ]\n
        [ --idle | --resume]"
    );
}

pub struct Settings {
    //<Tz> {
    pub full: bool,
    pub multi_timeline: bool,
    pub class_arg: String,
    pub interval: Interval,                      //<Tz>,
    pub class_mappings: HashMap<String, String>, //<Tz>,
}

impl Settings {
    fn new() -> Self {
        Self {
            full: false,
            multi_timeline: false,
            class_arg: String::from(""),
            interval: Interval::default(),
            class_mappings: HashMap::from([
                (String::from("Chromium-browser"), String::from("chromium")),
                (String::from("steam_app_813230"), String::from("steam")),
                (String::from("Unity"), String::from("unity")),
                (String::from("Alacritty"), String::from("alacritty")),
                (String::from("Slack"), String::from("slack")),
                (String::from("plasticx"), String::from("plastic")),
                (String::from("gcr-prompter"), String::from("keyring")),
                (
                    String::from(".blueman-manager-wrapped"),
                    String::from("blueman"),
                ),
                (
                    String::from("com.github.wwmm.easyeffects"),
                    String::from("easyeffects"),
                ),
                (String::from("org.gnome.Nautilus"), String::from("nautilus")),
                (String::from("org.pwmt.zathura"), String::from("zathura")),
                (
                    String::from("Xdg-desktop-portal-gtk"),
                    String::from("xdg-desktop-portal-gtk"),
                ),
            ]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval {
    //<Tz> {
    Days { days: u64 },
    // Dates {
    //     start: chrono::Date<Tz>,
    //     end: chrono::Date<Tz>,
    // },
    // DateTimes {
    //     start: chrono::DateTime<Tz>,
    //     end: chrono::DateTime<Tz>,
    // },
    // Timestamps {
    //     start: u64,
    //     end: u64,
    // },
}

impl Default for Interval {
    fn default() -> Self {
        Interval::Days { days: 1 }
    }
}

impl Interval {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn set_days(&mut self, value: u64) {
        match self {
            Interval::Days { days } => *days = value.max(1),
        }
    }

    pub fn date_str(&self) -> String {
        match *self {
            Interval::Days { days } => {
                let end = Utc::now().date_naive();
                let start = end - TimeDelta::days((days - 1) as i64);

                if days == 1 {
                    end.format("%Y-%m-%d").to_string()
                } else {
                    format!("{} - {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"))
                }
            }
        }
    }
}
