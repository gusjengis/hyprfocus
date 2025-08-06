use std::{collections::HashMap, error::Error, path::PathBuf};

use chrono::{Local, Timelike};
use csv::Reader;

use crate::Settings;

pub fn compute_durations(
    path: PathBuf,
    settings: &Settings,
) -> Result<(Vec<(String, u64)>, u64), Box<dyn Error>> {
    let mut map: HashMap<String, u64> = HashMap::new();
    let mut rdr = Reader::from_path(path)?;

    let mut total = 0;
    let mut last_timestamp = None;
    let mut last_class: Option<String> = None;
    let mut last_title: Option<String> = None;

    for result in rdr.records() {
        let record = result?;
        let timestamp: u64 = (record[0].parse::<i64>()?) as u64;
        let class = record[1].to_string();
        let title = record[2].to_string();

        if class == "SYSTEM" {
            match title.as_str() {
                "boot" => {
                    last_timestamp = None;
                    last_class = None;
                    last_title = None;
                }
                "resume" => {
                    last_timestamp = Some(timestamp);
                }
                "shutdown" | "idle" => {
                    add_interval_to_map(
                        last_timestamp,
                        timestamp,
                        last_class.as_ref(),
                        last_title.as_ref(),
                        settings,
                        &mut total,
                        &mut map,
                    );
                    last_timestamp = None;
                }
                _ => {}
            }
        } else {
            add_interval_to_map(
                last_timestamp,
                timestamp,
                last_class.as_ref(),
                last_title.as_ref(),
                settings,
                &mut total,
                &mut map,
            );

            last_timestamp = Some(timestamp);
            last_class = Some(class.clone());
            last_title = Some(title.clone());
            if settings.full {
                last_title = Some(format!("{class}: {title}"));
            }
        }
    }

    let timestamp = chrono::Local::now().timestamp_millis() as u64;
    add_interval_to_map(
        last_timestamp,
        timestamp,
        last_class.as_ref(),
        last_title.as_ref(),
        settings,
        &mut total,
        &mut map,
    );

    let mut vec: Vec<(String, u64)> = map.into_iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1));
    Ok((vec, total))
}

fn add_interval_to_map(
    last_timestamp: Option<u64>,
    end: u64,
    last_class: Option<&String>,
    last_title: Option<&String>,
    settings: &Settings,
    total: &mut u64,
    map: &mut HashMap<String, u64>,
) {
    if let (Some(start), Some(class), Some(title)) = (last_timestamp, last_class, last_title) {
        let duration = (end - start) as u64;
        *total += duration;

        if settings.full || class == &settings.class_arg {
            *map.entry(title.clone()).or_default() += duration;
        } else if settings.class_arg == "" {
            *map.entry(class.clone()).or_default() += duration;
        }
    }
}

pub fn timeline(
    path: &PathBuf,
    width: usize,
    settings: &Settings,
    label: Option<&String>,
) -> Vec<(String, i64, i64, bool, bool)> {
    let ms_per_day = 86400000;
    let ms_per_section = ms_per_day / width as i64;
    let midnight = Local::now()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    let starting_ms = midnight.timestamp_millis();
    let mut sections: Vec<(String, i64, i64, bool, bool)> =
        vec![(String::from(""), 0, 0, false, false); width];

    let mut rdr = Reader::from_path(path).unwrap();

    let mut last_timestamp: Option<i64> = None;
    let mut last_class: Option<String> = None;
    let mut last_title: Option<String> = None;

    for result in rdr.records() {
        let record = result.unwrap();
        let timestamp: i64 = record[0].parse().unwrap();
        let class = record[1].to_string();
        let title = record[2].to_string();

        if class == "SYSTEM" {
            match title.as_str() {
                "boot" => {
                    last_timestamp = None;
                    last_class = None;
                    last_title = None;
                }
                "resume" => {
                    last_timestamp = Some(timestamp);
                }
                "shutdown" | "idle" => {}
                _ => {
                    assign_interval_to_section(
                        last_timestamp,
                        timestamp,
                        last_class.as_ref(),
                        last_title.as_ref(),
                        starting_ms,
                        ms_per_section,
                        settings,
                        label,
                        &mut sections,
                    );
                    last_timestamp = None;
                }
            }
        } else {
            assign_interval_to_section(
                last_timestamp,
                timestamp,
                last_class.as_ref(),
                last_title.as_ref(),
                starting_ms,
                ms_per_section,
                settings,
                label,
                &mut sections,
            );
            last_timestamp = Some(timestamp);
            last_class = Some(class);
            last_title = Some(title);
        }
    }

    let timestamp = chrono::Local::now().timestamp_millis();
    assign_interval_to_section(
        last_timestamp,
        timestamp,
        last_class.as_ref(),
        last_title.as_ref(),
        starting_ms,
        ms_per_section,
        settings,
        label,
        &mut sections,
    );

    sections
}

fn assign_interval_to_section(
    last_timestamp: Option<i64>,
    timestamp: i64,
    last_class: Option<&String>,
    last_title: Option<&String>,
    starting_ms: i64,
    ms_per_section: i64,
    settings: &Settings,
    label: Option<&String>,
    sections: &mut Vec<(String, i64, i64, bool, bool)>,
) {
    if let (Some(start), Some(class_name), Some(title)) = (last_timestamp, last_class, last_title) {
        if settings.multi_timeline && (label.unwrap() == &key(settings, class_name, title))
            || !settings.multi_timeline
                && (settings.class_arg == "" || settings.full || &settings.class_arg == class_name)
        {
            let edge_detection_padding = (ms_per_section as f64 / 10.0) as i64;
            let start_index = section_index(starting_ms, ms_per_section, start);
            let end_index = section_index(starting_ms, ms_per_section, timestamp);
            for i in start_index..end_index + 1 {
                let section_start = starting_ms + ms_per_section * i as i64;
                let section_end = starting_ms + ms_per_section * (i as i64 + 1);
                let contribution = section_end.min(timestamp) - section_start.max(start);
                if section_start + edge_detection_padding >= start {
                    sections[i].3 = true;
                }
                if section_end - edge_detection_padding <= timestamp {
                    sections[i].4 = true;
                }
                let key = key(settings, class_name, title);
                sections[i].2 += contribution;
                if sections[i].0 == key {
                    sections[i].1 += contribution;
                } else {
                    sections[i].1 -= contribution;
                    if sections[i].1 < 0 {
                        sections[i].0 = key.clone();
                        sections[i].1 *= -1;
                    }
                }
            }
        }
    }
}

fn section_index(starting_ms: i64, ms_per_section: i64, timestamp: i64) -> usize {
    ((timestamp - starting_ms) / ms_per_section) as usize
}

fn key(settings: &Settings, last_class: &String, last_title: &String) -> String {
    if settings.full {
        format!("{last_class}: {last_title}")
    } else if settings.class_arg == "" {
        last_class.clone()
    } else {
        last_title.clone()
    }
}
