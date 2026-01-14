use std::collections::HashMap;

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
        // let mut mappings = HashMap<String, String>::new();
        // load config file at ~/.config/hypr/hyprlog.conf
        // split into lines and parse each line
        // store all parsed configuration is self
    }
}
