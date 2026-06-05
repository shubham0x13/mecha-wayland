use crate::time::{self, Precision};

#[derive(Debug)]
pub struct ClockChanged;
impl app::Event for ClockChanged {}

#[derive(Debug)]
pub struct ClockUpdate(pub u32, pub u32, pub u32, pub u32, pub u32); // h, m, s, day, mon
impl app::Event for ClockUpdate {}

#[derive(Debug)]
pub struct ClockWidget {
    pub time_str: String,
    pub format_24h: bool,
}

impl ClockWidget {
    pub fn new() -> Self {
        let (h, m, _, _, _) = time::local_time();
        let mut w = Self {
            time_str: String::new(),
            format_24h: true,
        };
        w.time_str = w.formatted_text(h, m);
        w
    }

    pub fn precision(&self) -> Precision {
        Precision::Minutes
    }

    fn formatted_text(&self, h: u32, m: u32) -> String {
        let hour = if self.format_24h {
            h
        } else {
            ((h + 11) % 12) + 1
        };

        if self.format_24h {
            format!("{:02}:{:02}", hour, m)
        } else {
            let am_pm = if h < 12 { "AM" } else { "PM" };
            format!("{:02}:{:02} {}", hour, m, am_pm)
        }
    }
}

pub fn module<AppState>() -> impl app::RegisteredModule<ClockWidget, AppState> {
    app::Module::new().on(|w: &mut ClockWidget, ev: &ClockUpdate| {
        let new = w.formatted_text(ev.0, ev.1);
        if w.time_str != new {
            w.time_str = new;
            return Some(ClockChanged);
        }
        None
    })
}
