use std::collections::HashMap;

use crate::interval::Interval;

pub struct Settings {
    //<Tz> {
    pub full: bool,
    pub multi_timeline: bool,
    pub class_arg: String,
    pub interval: Interval,                      //<Tz>,
    pub class_mappings: HashMap<String, String>, //<Tz>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            full: false,
            multi_timeline: false,
            class_arg: String::from(""),
            interval: Interval::default(),
            class_mappings: HashMap::from([
                // (String::from("Chromium-browser"), String::from("chromium")),
                (String::from("steam_app_813230"), String::from("steam")),
                // (String::from("steam_app_2357570"), String::from("Overwatch")),
                // ( String::from("steam_app_2050650"), String::from("Resident Evil 4"),),
                // (String::from("Unity"), String::from("unity")),
                // (String::from("Alacritty"), String::from("alacritty")),
                // (String::from("Slack"), String::from("slack")),
                // (String::from("plasticx"), String::from("plastic")),
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
                // (
                //     String::from("Xdg-desktop-portal-gtk"),
                //     String::from("xdg-desktop-portal-gtk"),
                // ),
            ]),
        }
    }
}
