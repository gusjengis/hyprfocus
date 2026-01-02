struct Data {
    logs: Vec<Log>,
}

struct Log {
    timestamp: u64,
    datum: Datum,
}

enum Datum {
    FocusChange { class: String, title: String },
    Idle,
    Resume,
    Boot,
    ShutdoWn,
}

pub struct Timeline {
    pub sections: Vec<Segment>,
    pub scale: u64, // ms per character
}

pub struct Segment {
    pub start: u64,
    pub end: u64,
    pub title: String,
}

pub struct Report {
    pub classes: Vec<Class>,
}

impl Report {
    pub fn new() -> Self {
        Report { classes: vec![] }
    }

    pub fn total_duration(&self) -> u64 {
        self.classes.iter().map(|c| c.total()).sum()
    }
}
pub struct Class {
    pub titles: Vec<Title>,
}

impl Class {
    pub fn new() -> Self {
        Class { titles: vec![] }
    }

    pub fn total(&self) -> u64 {
        self.titles.iter().map(|t| t.duration).sum()
    }
}

pub struct Title {
    pub title: String,
    pub duration: u64,
}
