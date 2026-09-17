use std::collections::VecDeque;
use std::sync::Mutex;

pub const HISTORY: usize = 64;

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub cpu_pct: f32,
    pub cpu_history: VecDeque<f32>,
    pub ram_used: u64,
    pub ram_total: u64,
    pub ram_history: VecDeque<f32>,
    pub disk_used: u64,
    pub disk_free: u64,
    pub disk_total: u64,
    pub net_down_bps: f64,
    pub net_up_bps: f64,
    pub net_down_history: VecDeque<f32>,
    pub net_up_history: VecDeque<f32>,
    pub mail_in: u32,
    pub mail_out: u32,
    pub mail_total: u32,
    pub mail_unread: u32,
    pub calendar_events: Vec<String>,
    pub weather_city: String,
    pub weather_temp_c: Option<f32>,
    pub weather_desc: String,
    pub weather_humidity: Option<f32>,
    pub weather_wind: Option<f32>,
    pub weather_code: Option<i32>,
    pub weather_is_day: bool,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            cpu_pct: 0.0,
            cpu_history: VecDeque::from(vec![0.0; HISTORY]),
            ram_used: 0,
            ram_total: 1,
            ram_history: VecDeque::from(vec![0.0; HISTORY]),
            disk_used: 0,
            disk_free: 0,
            disk_total: 1,
            net_down_bps: 0.0,
            net_up_bps: 0.0,
            net_down_history: VecDeque::from(vec![0.0; HISTORY]),
            net_up_history: VecDeque::from(vec![0.0; HISTORY]),
            mail_in: 0,
            mail_out: 0,
            mail_total: 0,
            mail_unread: 0,
            calendar_events: Vec::new(),
            weather_city: "Locating nearest city".to_string(),
            weather_temp_c: None,
            weather_desc: String::new(),
            weather_humidity: None,
            weather_wind: None,
            weather_code: None,
            weather_is_day: true,
        }
    }
}

impl Snapshot {
    pub fn ram_pct(&self) -> f32 {
        if self.ram_total == 0 {
            0.0
        } else {
            (self.ram_used as f64 / self.ram_total as f64) as f32 * 100.0
        }
    }

    pub fn disk_used_pct(&self) -> f32 {
        if self.disk_total == 0 {
            0.0
        } else {
            (self.disk_used as f64 / self.disk_total as f64) as f32 * 100.0
        }
    }

    pub fn push(buf: &mut VecDeque<f32>, value: f32) {
        if buf.len() >= HISTORY {
            buf.pop_front();
        }
        buf.push_back(value.clamp(0.0, f32::MAX));
    }
}

pub type SharedSnapshot = Mutex<Snapshot>;
