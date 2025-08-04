use chrono::Local;
use csv::Reader;
use std::{
    collections::HashMap,
    env::home_dir,
    error::Error,
    fs::{create_dir_all, metadata},
    path::PathBuf,
};
use terminal_size::Width;

pub fn render_log() {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let mut dir: PathBuf = home_dir().expect("could not get home dir");
    dir.push(".local/share/hyprfocus");
    let path = dir.join(format!("{}.csv", date_str));

    create_dir_all(dir).expect("failed to create data directory");
    let file_exists = metadata(&path).is_ok();
    if !file_exists {
        println!("{} not found, start hyprfocusd.", path.to_string_lossy());
        return;
    }

    if let Ok((basic_view, total)) = sorted_classes(path) {
        println!("{}\nUptime: {}", date_str, format_duration(total));
        print_table(basic_view, total);
    } else {
        println!("An error occurred while parsing the log.");
    }
}

pub fn render_class_log(class: &str) {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let mut dir: PathBuf = home_dir().expect("could not get home dir");
    dir.push(".local/share/hyprfocus");
    let path = dir.join(format!("{}.csv", date_str));

    create_dir_all(dir).expect("failed to create data directory");
    let file_exists = metadata(&path).is_ok();
    if !file_exists {
        println!("{} not found, start hyprfocusd.", path.to_string_lossy());
        return;
    }

    if let Ok((basic_view, total)) = sorted_titles(path, class) {
        if (basic_view.len() == 0) {
            println!("Class \"{class}\" not found in log.");
            return;
        }
        println!("{}\nUptime: {}", date_str, format_duration(total));
        print_table(basic_view, total);
    } else {
        println!("An error occurred while parsing the log.");
    }
}

fn print_table(rows: Vec<(String, u32)>, total: u32) {
    // Step 1: Determine the max width of the "Class" column
    let class_header = "Class";
    let mut max_class_width = rows
        .iter()
        .map(|(class, _)| class.len())
        .max()
        .unwrap_or(0)
        .max(class_header.len());

    let max_string_length = match terminal_size::terminal_size() {
        Some((Width(w), _)) => w as usize - 20,
        None => 100,
    };
    max_class_width = max_class_width.min(max_string_length);
    // Step 2: Prepare header and separator
    println!(
        "{:<width$} {:>10} {:>8}",
        class_header,
        "Time",
        "Percent",
        width = max_class_width
    );

    let total_width = max_class_width + 10 + 8 + 2; // +2 for the spaces between columns
    println!("{:-<1$}", "", total_width);

    // Step 3: Print each row using dynamic width
    for (class, duration) in rows {
        let percent = 100.0 * (duration as f64 / total as f64);
        println!(
            "{:<width$} {:>10} {:>7.2}%",
            truncate_string(&class, max_string_length),
            format_duration(duration),
            percent,
            width = max_class_width
        );
    }
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

fn sorted_classes(path: PathBuf) -> Result<(Vec<(String, u32)>, u32), Box<dyn Error>> {
    let mut map: HashMap<String, u32> = HashMap::new();
    let mut rdr = Reader::from_path(path)?;

    let mut total = 0;
    let mut last_timestamp = None;
    let mut last_class: Option<String> = None;

    for result in rdr.records() {
        let record = result?;
        let timestamp: i64 = record[0].parse()?;
        let class = record[1].to_string();
        let title = record[2].to_string();

        if class == "SYSTEM" {
            match title.as_str() {
                "boot" => {
                    last_timestamp = None;
                    last_class = None;
                }
                "resume" => {
                    last_timestamp = Some(timestamp);
                }
                "shutdown" | "idle" => {
                    if let (Some(last), Some(class_name)) = (last_timestamp, last_class.as_ref()) {
                        let duration = (timestamp - last) as u32;
                        total += duration;
                        *map.entry(class_name.clone()).or_default() += duration;
                    }
                }
                _ => {}
            }
        } else {
            if let (Some(last), Some(class_name)) = (last_timestamp, last_class.as_ref()) {
                let duration = (timestamp - last) as u32;
                total += duration;
                *map.entry(class_name.clone()).or_default() += duration;
            }

            last_timestamp = Some(timestamp);
            last_class = Some(class);
        }
    }

    let timestamp = chrono::Local::now().timestamp_millis();
    let duration = (timestamp - last_timestamp.unwrap()) as u32;
    total += duration;
    *map.entry(last_class.unwrap().clone()).or_default() += duration;

    let mut vec: Vec<(String, u32)> = map.into_iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1));
    Ok((vec, total))
}

fn sorted_titles(
    path: PathBuf,
    class_arg: &str,
) -> Result<(Vec<(String, u32)>, u32), Box<dyn Error>> {
    let mut map: HashMap<String, u32> = HashMap::new();
    let mut rdr = Reader::from_path(path)?;

    let mut total = 0;
    let mut last_timestamp = None;
    let mut last_class: Option<String> = None;
    let mut last_title: Option<String> = None;

    for result in rdr.records() {
        let record = result?;
        let timestamp: i64 = record[0].parse()?;
        let class = record[1].to_string();
        let title = record[2].to_string();

        if class == "SYSTEM" {
            match title.as_str() {
                "boot" => {
                    last_timestamp = None;
                    last_class = None;
                }
                "resume" => {
                    last_timestamp = Some(timestamp);
                }
                "shutdown" | "idle" => {
                    if let (Some(last), Some(class_name), Some(key)) =
                        (last_timestamp, last_class.as_ref(), last_title.as_ref())
                    {
                        let duration = (timestamp - last) as u32;
                        total += duration;
                        if *class_name == class_arg {
                            *map.entry(key.clone()).or_default() += duration;
                        }
                    }
                }
                _ => {}
            }
        } else {
            if let (Some(last), Some(class_name), Some(key)) =
                (last_timestamp, last_class.as_ref(), last_title.as_ref())
            {
                let duration = (timestamp - last) as u32;
                total += duration;
                if *class_name == class_arg {
                    *map.entry(key.clone()).or_default() += duration;
                }
            }

            last_timestamp = Some(timestamp);
            last_class = Some(class.clone());
            last_title = Some(title.clone());
        }
    }

    if let (Some(last), Some(class_name), Some(key)) =
        (last_timestamp, last_class.as_ref(), last_title.as_ref())
    {
        let timestamp = chrono::Local::now().timestamp_millis();
        let duration = (timestamp - last_timestamp.unwrap()) as u32;
        total += duration;
        if *class_name == class_arg {
            *map.entry(key.clone()).or_default() += duration;
        }
    }

    let mut vec: Vec<(String, u32)> = map.into_iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1));
    Ok((vec, total))
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
