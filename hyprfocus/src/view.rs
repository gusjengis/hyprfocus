use chrono::Local;
use csv::Reader;
use std::{
    collections::HashMap,
    env::home_dir,
    error::Error,
    fs::{create_dir_all, metadata},
    path::PathBuf,
};

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
        println!("{:<20} {:>10} {:>8}", "Class", "Time", "Percent");
        println!("{:-<40}", ""); // horizontal rule

        for (class, duration) in basic_view {
            let percent = 100.0 * (duration as f64 / total as f64);
            println!(
                "{:<20} {:>10} {:>7.2}%",
                class,
                format_duration(duration),
                percent
            );
        }
    } else {
        println!("An error occurred while parsing the log.");
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
                "shutdown" => {
                    if let (Some(last), Some(class_name)) = (last_timestamp, last_class.as_ref()) {
                        let duration = (timestamp - last) as u32;
                        total += duration;
                        *map.entry(class_name.clone()).or_default() += duration;
                    }
                }
                "idle" => {
                    if let (Some(last), Some(class_name)) = (last_timestamp, last_class.as_ref()) {
                        let duration = (timestamp - last) as u32;
                        total += duration;
                        *map.entry(class_name.clone()).or_default() += duration;
                    }
                }
                "resume" => {
                    last_timestamp = Some(timestamp);
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
