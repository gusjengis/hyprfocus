use crate::log_parsing::{compute_durations, timeline};
use chrono::Local;
use colored::{Color, Colorize};
use std::fmt::Write;
use std::{
    collections::HashMap,
    env::home_dir,
    fs::{create_dir_all, metadata},
    path::PathBuf,
};
use terminal_size::Width;

pub fn daily_log_path(date_str: &str) -> Option<PathBuf> {
    let mut dir: PathBuf = home_dir().expect("could not get home dir");
    dir.push(".local/share/hyprfocus");
    let path = dir.join(format!("{}.csv", date_str));

    create_dir_all(dir).expect("failed to create data directory");
    let file_exists = metadata(&path).is_ok();
    if !file_exists {
        println!("{} not found, start hyprfocusd.", path.to_string_lossy());
        return None;
    }
    return Some(path);
}

pub fn render_log(class_arg: &str, multi_timeline: bool) {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    if let Some(path) = daily_log_path(&date_str) {
        match compute_durations(path, class_arg) {
            Ok((durations, total)) => {
                if durations.is_empty() {
                    if class_arg == "" {
                        println!("Empty log.");
                    } else {
                        println!("Class \"{class_arg}\" not found in log.");
                    }
                    return;
                }

                let colors = key_to_color_map(&durations);
                let labels: Vec<String> = durations.iter().map(|(s, _)| s.clone()).collect();
                print_header(&date_str);
                render_timelines(&colors, class_arg, multi_timeline, labels);
                print_table(durations, total, &colors);
            }
            Err(e) => {
                eprintln!("Failed to compute durations: {e:?}");
            }
        }
    };
}

const STRIKE_ON: &str = "\x1b[9m";
const STRIKE_OFF: &str = "\x1b[29m";
const FANCY_TIMELINE: bool = true;

pub fn render_timelines(
    colors: &HashMap<String, Color>,
    class_arg: &str,
    multi_timeline: bool,
    labels: Vec<String>,
) {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    if let Some(path) = daily_log_path(&date_str) {
        let width = terminal_width();
        if !multi_timeline {
            render_timeline(&date_str, colors, &class_arg, &"", multi_timeline);
        } else {
            for label in labels {
                if label.len() == 0 {
                    continue;
                }
                let mut title = "";
                if class_arg == "*" {
                    title = label.split_once(':').unwrap().1;
                }
                render_timeline(&date_str, colors, &label, &title, multi_timeline);
            }
            // render_timeline(&date_str, colors, &class_arg, &"", false);
        }
    }
}

pub fn render_timeline(
    date_str: &str,
    colors: &HashMap<String, Color>,
    class_arg: &str,
    title_arg: &str,
    multi_timeline: bool,
) {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    if let Some(path) = daily_log_path(&date_str) {
        let width = terminal_width();
        // println!("{}", key);
        let sections = timeline(&path, width, &class_arg, &title_arg);
        let mut timeline_string = String::from("");

        for section_data in sections {
            let key = match multi_timeline {
                false => &section_data.0,
                true => class_arg,
            };
            if let Some(color) = colors.get(key) {
                let ch = choose_character(section_data).to_string();
                let glyph = if *color == Color::Black {
                    // plain em dash (or space) for gaps
                    if FANCY_TIMELINE {
                        " ".strikethrough().bold().white().to_string()
                    } else {
                        "—".white().to_string()
                    }
                } else {
                    format!("{}", ch.color(*color))
                };

                write!(&mut timeline_string, "{}{}{}", STRIKE_ON, glyph, STRIKE_OFF).unwrap();
            }
        }
        println!("{timeline_string}\n");
    }
}

fn choose_character(section_data: (String, i64, i64, bool, bool)) -> char {
    let width = terminal_width();
    let ms_per_day = 86400000;
    let ms_per_section = ms_per_day / width as i64;
    let fullness = section_data.2 as f64 / ms_per_section as f64;
    if FANCY_TIMELINE {
        if section_data.3 && section_data.4 {
            // there is activity near both the left and right side of a section
            return '█';
        } else if section_data.3 {
            // there is activity near the left side of a section
            return match fullness {
                f64::MIN..=0.00 => ' ',
                0.0..=0.1250000 => '▏',
                0.125..=0.25000 => '▎',
                0.25..=0.375000 => '▍',
                0.375..=0.50000 => '▌',
                0.5..=0.6250000 => '▋',
                0.625..=0.75000 => '▊',
                0.75..=f64::MAX => '█',
                _ => '—',
            };
        } else if section_data.4 {
            // there is activity near the right side of a section
            return match fullness {
                f64::MIN..=0.00 => ' ',
                0.0..=0.1250000 => '🮇',
                0.125..=0.25000 => '🮈',
                0.25..=0.375000 => '▐',
                0.375..=0.50000 => '🮉',
                0.5..=0.6250000 => '🮊',
                0.625..=0.75000 => '🮋',
                0.75..=f64::MAX => '█',
                _ => ' ',
            };
        } else {
            return match fullness {
                f64::MIN..=0.00 => ' ',
                0.00..=0.333333 => '│',
                0.333333..=0.66 => '┃',
                0.66..=f64::MAX => '█',
                _ => ' ',
            };
        }
    } else {
        if fullness == 0.0 {
            return ' ';
        }
        return '█';
    }
}

fn print_header(date_str: &str) {
    let term_width = terminal_width();

    // padding on the text inside the box
    let inner_width = date_str.len() + 2;
    let box_width = inner_width + 2; // +2 for the two vertical borders

    // Fallback if the terminal is too narrow
    if box_width > term_width {
        println!("{}", date_str);
        return;
    }

    let start_column = (term_width - box_width) / 2;
    let pad = " ".repeat(start_column);

    println!("{}╭{}╮", pad, "─".repeat(inner_width));
    println!("{}│ {} │", pad, date_str);
    println!("{}╰{}╯", pad, "─".repeat(inner_width));
    println!("\n");
}

fn print_table(rows: Vec<(String, u32)>, total: u32, colors: &HashMap<String, Color>) {
    let mut max_class_width = rows.iter().map(|(class, _)| class.len()).max().unwrap_or(0);

    let max_string_length = terminal_width() - 20;
    max_class_width = max_class_width.min(max_string_length);

    let total_width = max_class_width + 10 + 8 + 2; // +2 for the spaces between columns
    let left_padding = (terminal_width() - total_width) / 2;

    println!("");

    let mut total_percentage = 0.0;
    let mut total_duration = 0;
    for (class, duration) in rows {
        total_duration += duration;
        let percent = 100.0 * (duration as f64 / total as f64);
        total_percentage += percent;
        let color = colors.get(&class).unwrap();
        println!(
            "{}{:<width$} {:>10} {:>7.2}%",
            " ".repeat(left_padding),
            truncate_string(&class, max_string_length).color(*color),
            format_duration(duration),
            percent,
            width = max_class_width
        );
    }

    println!(
        "\n{}{:<width$} {:>10} {:>7.2}%",
        " ".repeat(left_padding),
        truncate_string(&"Total", max_string_length).bold(),
        format_duration(total_duration),
        total_percentage,
        width = max_class_width
    );
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        ".".repeat(max_len) // handles silly small max_len
    }
}

fn format_duration(ms: u32) -> String {
    let secs = ms / 1000;
    let minutes = secs / 60;
    let seconds = secs % 60;
    if minutes >= 60 {
        let hours = minutes / 60;
        let rem_minutes = minutes % 60;
        format!("{:02}:{:02}:{:02}", hours, rem_minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

fn terminal_width() -> usize {
    return match terminal_size::terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 120,
    };
}

pub fn color_from_index(index: usize) -> Color {
    return match index {
        0 => Color::Green,
        1 => Color::Red,
        2 => Color::Blue,
        3 => Color::Magenta,
        4 => Color::Yellow,
        5 => Color::Cyan,
        6 => Color::BrightRed,
        7 => Color::BrightGreen,
        8 => Color::BrightYellow,
        9 => Color::BrightBlue,
        10 => Color::BrightMagenta,
        11 => Color::BrightCyan,
        _ => Color::White,
    };
}

fn key_to_color_map(list: &Vec<(String, u32)>) -> HashMap<String, Color> {
    let mut res: HashMap<String, Color> = HashMap::new();
    res.insert(String::from(""), Color::Black);
    let mut color_index = 0;
    for entry in list {
        if !res.contains_key(&entry.0) {
            res.insert(entry.0.clone(), color_from_index(color_index));
            color_index += 1;
        }
    }

    return res;
}
