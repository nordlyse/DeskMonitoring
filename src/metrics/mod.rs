use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::snapshot::{SharedSnapshot, Snapshot};

mod calendar;
mod mail;
mod system;
mod weather;

pub struct Collector {
    system: system::SystemCollector,
}

impl Collector {
    pub fn add() -> Self {
        Self {
            system: system::SystemCollector::add(),
        }
    }

    pub fn refresh_local(&mut self, snapshot: &SharedSnapshot) {
        self.system.refresh(snapshot);
    }
}

pub fn spawn_remote_loop(config: std::sync::Arc<std::sync::Mutex<Config>>, snapshot: std::sync::Arc<SharedSnapshot>) {
    std::thread::Builder::new()
        .name("desk-monitor-remote".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            let mut ticks = 0u64;
            loop {
                let cfg = config.lock().map(|guard| guard.clone()).unwrap_or_default();
                rt.block_on(async {
                    if ticks % 15 == 0 {
                        weather::refresh(&snapshot).await;
                    }
                    calendar::refresh(&cfg, &snapshot);
                    mail::refresh(&cfg, &snapshot).await;
                });
                ticks = ticks.saturating_add(1);
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        })
        .expect("remote metrics thread");
}

pub fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;
    let n = bytes as f64;
    if n >= TIB {
        format!("{:.1} TiB", n / TIB)
    } else if n >= GIB {
        format!("{:.1} GiB", n / GIB)
    } else if n >= MIB {
        format!("{:.1} MiB", n / MIB)
    } else if n >= KIB {
        format!("{:.1} KiB", n / KIB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_bps(bps: f64) -> String {
    format_bytes(bps.max(0.0) as u64) + "/s"
}

pub fn walk_ics_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_ics_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ics") {
            out.push(path);
        }
        if out.len() > 80 {
            return;
        }
    }
}

pub fn apply_mail(snapshot: &SharedSnapshot, incoming: u32, outgoing: u32, unread: u32) {
    if let Ok(mut snap) = snapshot.lock() {
        snap.mail_in = incoming;
        snap.mail_out = outgoing;
        snap.mail_total = incoming.saturating_add(outgoing);
        snap.mail_unread = unread;
    }
}

pub fn apply_calendar(snapshot: &SharedSnapshot, events: Vec<String>) {
    if let Ok(mut snap) = snapshot.lock() {
        snap.calendar_events = events;
    }
}

pub fn apply_weather(
    snapshot: &SharedSnapshot,
    city: String,
    temp: Option<f32>,
    desc: String,
    humidity: Option<f32>,
    wind: Option<f32>,
    code: Option<i32>,
    is_day: bool,
) {
    if let Ok(mut snap) = snapshot.lock() {
        snap.weather_city = city;
        snap.weather_temp_c = temp;
        snap.weather_desc = desc;
        snap.weather_humidity = humidity;
        snap.weather_wind = wind;
        snap.weather_code = code;
        snap.weather_is_day = is_day;
    }
}

pub fn copy_snapshot(snapshot: &SharedSnapshot) -> Snapshot {
    snapshot
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}
