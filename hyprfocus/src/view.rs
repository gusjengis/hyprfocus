use crate::Settings;
use crate::log_parsing::{MS_PER_DAY, compute_durations, timeline};
use crate::log_reader::LogReader;
use colored::{Color, Colorize};
use std::collections::HashMap;
use std::fmt::Write;
use terminal_size::Width;

pub fn render_log(settings: &Settings) {
    let mut reader = LogReader::new(settings);
    if !reader.is_empty() {
        match compute_durations(&mut reader, settings) {
            Ok((durations, total)) => {
                if durations.is_empty() {
                    if &settings.class_arg == "" {
                        println!("Empty log.");
                    } else {
                        println!("Class \"{}\" not found in log.", &settings.class_arg);
                    }
                    return;
                }

                let colors = key_to_color_map(&durations);
                let labels: Vec<String> = durations.iter().map(|(s, _)| s.clone()).collect();
                print_header(settings);
                render_timelines(&mut reader, &colors, labels, settings);
                print_table(durations, total, &colors);
            }
            Err(e) => {
                eprintln!("Failed to compute durations: {e:?}");
            }
        }
    } else {
        println!(
            "Log files not found in the following interval.\n{:?}",
            settings.interval
        );
    }
}

fn print_header(settings: &Settings) {
    let date_str = settings.interval.date_str();
    let term_width = terminal_width();

    let inner_width = date_str.len() + 2;
    let box_width = inner_width + 2;

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

const STRIKE_ON: &str = "\x1b[9m";
const STRIKE_OFF: &str = "\x1b[29m";
const FANCY_TIMELINE: bool = true;
const CUTOFF: usize = usize::MAX; // not doing anything but the setting is here

pub fn render_timelines(
    reader: &mut LogReader,
    colors: &HashMap<String, Color>,
    labels: Vec<String>,
    settings: &Settings,
) {
    if !settings.multi_timeline {
        render_timeline(reader, colors, settings, None);
    } else {
        let mut count = 0;
        for label in labels {
            if label.len() == 0 {
                continue;
            }
            if count >= CUTOFF {
                break;
            }
            render_timeline(reader, colors, settings, Some(&label));
            count += 1;
        }
    }
}

pub fn render_timeline(
    reader: &mut LogReader,
    colors: &HashMap<String, Color>,
    settings: &Settings,
    label: Option<&String>,
) {
    let width = terminal_width();
    let sections = timeline(reader, width, &settings, label);
    let mut timeline_string = String::from("");

    for section_data in sections {
        let key = match settings.multi_timeline {
            false => &section_data.0,
            true => label.unwrap(),
        };
        if let Some(color) = colors.get(key) {
            let ch = choose_character(section_data, settings).to_string();
            let glyph = if *color == Color::Black {
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

fn choose_character(section_data: (String, i64, i64, bool, bool), settings: &Settings) -> char {
    let width = terminal_width();
    let ms_per_section = match settings.interval {
        crate::Interval::Days { days } => (days as i64 * MS_PER_DAY) as f64 / width as f64,
    };
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

fn print_table(rows: Vec<(String, u64)>, total: u64, colors: &HashMap<String, Color>) {
    let mut max_class_width = rows.iter().map(|(class, _)| class.len()).max().unwrap_or(0);

    let max_string_length = terminal_width() - 20;
    max_class_width = max_class_width.min(max_string_length);

    let total_width = max_class_width + 10 + 8 + 2; // +2 for the spaces between columns
    let left_padding = (terminal_width() - total_width) / 2;

    println!("");

    let mut total_percentage = 0.0;
    let mut total_duration = 0;
    let mut count = 0;
    for (class, duration) in rows {
        if count >= CUTOFF {
            break;
        }
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
        count += 1;
    }

    println!(
        "{}",
        format!(
            "\n{}{:<width$} {:>10} {:>7.2}%",
            " ".repeat(left_padding),
            truncate_string(&"Total", max_string_length),
            format_duration(total_duration),
            total_percentage,
            width = max_class_width
        )
        .bold()
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

pub fn format_duration(ms: u64) -> String {
    if ms < 1_000 {
        return format!("{ms}ms");
    }

    const SEC_PER_MIN: u64 = 60;
    const MIN_PER_HOUR: u64 = 60;
    const HOUR_PER_DAY: u64 = 24;
    const DAY_PER_YEAR: u64 = 365;

    let total_secs = ms / 1_000;

    let seconds = total_secs % SEC_PER_MIN;
    let total_mins = total_secs / SEC_PER_MIN;

    let minutes = total_mins % MIN_PER_HOUR;
    let total_hours = total_mins / MIN_PER_HOUR;

    let hours = total_hours % HOUR_PER_DAY;
    let total_days = total_hours / HOUR_PER_DAY;

    let days = total_days % DAY_PER_YEAR;
    let years = total_days / DAY_PER_YEAR;

    if years > 0 {
        format!(
            "{years}y {days}d {:02}:{:02}:{:02}",
            hours, minutes, seconds
        )
    } else if days > 0 {
        format!("{days}d {:02}:{:02}:{:02}", hours, minutes, seconds)
    } else if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
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
        8 => Color::BrightBlue,
        9 => Color::BrightMagenta,
        10 => Color::BrightYellow,
        11 => Color::BrightCyan,
        _ => Color::White,
    };
}

fn key_to_color_map(list: &Vec<(String, u64)>) -> HashMap<String, Color> {
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
