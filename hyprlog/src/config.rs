use directories::BaseDirs;
use std::{collections::HashMap, fs, path::PathBuf};

pub struct Config {
    pub class_mappings: HashMap<String, String>, //<Tz>,
}

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();
        config.parse_config();
        config
    }

    fn default() -> Self {
        Self {
            class_mappings: HashMap::new(),
        }
    }

    fn parse_config(&mut self) {
        let config_path = BaseDirs::new()
            .map(|b| b.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::new())
            .join("hypr/hyprlog.conf");
        if config_path.exists() {
            let config = fs::read_to_string(config_path).expect("Failed to read hyprlog.conf.");
            let mut line_number = 0;
            for line in config.lines() {
                line_number += 1;
                if line.starts_with('#') {
                    continue;
                }
                // removes comments and leading or trailing whitespace
                let trimmed_line = line.trim().split('#').next().unwrap();

                if trimmed_line.starts_with("class_alias") {
                    let equals_split: Vec<&str> = trimmed_line.split("=").collect();
                    if equals_split.len() == 1 {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Missing '='.",
                            line_number
                        );
                        continue;
                    } else if equals_split.len() > 2 {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Multiple '='.",
                            line_number
                        );
                        continue;
                    }
                    let values: Vec<&str> = equals_split[1].split(",").collect();
                    if values.len() == 1 {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Missing ','.",
                            line_number
                        );
                        continue;
                    } else if equals_split.len() > 2 {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Multiple ',', there should only be two values per alias.",
                            line_number
                        );
                        continue;
                    }

                    let class = values[0]
                        .trim()
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'));
                    if class.is_none() {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Class string is not properly quoted.",
                            line_number
                        );
                        continue;
                    }
                    let alias = values[1]
                        .trim()
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'));
                    if alias.is_none() {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Alias string is not properly quoted.",
                            line_number
                        );
                        continue;
                    }

                    self.class_mappings
                        .insert(class.unwrap().to_string(), alias.unwrap().to_string());
                } else {
                    if !line.is_empty() {
                        println!(
                            "Failed to parse hyprlog.conf at line {}: Unknown symbol {}",
                            line_number,
                            line.split_whitespace().next().unwrap()
                        );
                    }
                }
                continue;
            }
        }
    }
}
