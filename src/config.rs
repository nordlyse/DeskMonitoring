use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl Position {
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaletteKind {
    Matrix,
    Turquoise,
    Blue,
    Pink,
    Yellow,
}

impl PaletteKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Matrix => "Matrix green",
            Self::Turquoise => "Turquoise",
            Self::Blue => "Blue",
            Self::Pink => "Pink",
            Self::Yellow => "Yellow",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Panels {
    #[serde(default = "default_true")]
    pub cpu: bool,
    #[serde(default = "default_true")]
    pub ram: bool,
    #[serde(default = "default_true")]
    pub disk: bool,
    #[serde(default = "default_true")]
    pub network: bool,
    #[serde(default = "default_true")]
    pub mail: bool,
    #[serde(default = "default_true")]
    pub calendar: bool,
    #[serde(default = "default_true")]
    pub weather: bool,
}

impl Default for Panels {
    fn default() -> Self {
        Self {
            cpu: true,
            ram: true,
            disk: true,
            network: true,
            mail: true,
            calendar: true,
            weather: true,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub position: Position,
    pub palette: PaletteKind,
    #[serde(default)]
    pub panels: Panels,
    #[serde(default)]
    pub imap_host: String,
    #[serde(default = "default_imap_port")]
    pub imap_port: u16,
    #[serde(default)]
    pub imap_user: String,
    #[serde(default)]
    pub imap_password: String,
    #[serde(default = "default_inbox")]
    pub imap_inbox: String,
    #[serde(default = "default_sent")]
    pub imap_sent: String,
    #[serde(default)]
    pub calendar_ics: String,
}

fn default_imap_port() -> u16 {
    993
}

fn default_inbox() -> String {
    "INBOX".to_string()
}

fn default_sent() -> String {
    "Sent".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            position: Position::Right,
            palette: PaletteKind::Matrix,
            panels: Panels::default(),
            imap_host: String::new(),
            imap_port: default_imap_port(),
            imap_user: String::new(),
            imap_password: String::new(),
            imap_inbox: default_inbox(),
            imap_sent: default_sent(),
            calendar_ics: String::new(),
        }
    }
}

impl Config {
    pub fn mail_ready(&self) -> bool {
        !self.imap_host.trim().is_empty() && !self.imap_user.trim().is_empty()
    }
}

pub fn config_path() -> Result<PathBuf, String> {
    let root = dirs::config_dir().ok_or_else(|| "No config directory on this system".to_string())?;
    Ok(root.join("desk-monitoring").join("config.toml"))
}

pub fn load_config() -> Option<Config> {
    let path = config_path().ok()?;
    let raw = std::fs::read_to_string(path).ok()?;
    toml::from_str(&raw).ok()
}

pub fn write_config(config: &Config) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, raw).map_err(|e| e.to_string())?;
    store_live(config);
    Ok(())
}

static LIVE: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();

pub fn live_slot() -> Arc<Mutex<Config>> {
    LIVE.get_or_init(|| Arc::new(Mutex::new(load_config().unwrap_or_default())))
        .clone()
}

pub fn store_live(config: &Config) {
    if let Ok(mut guard) = live_slot().lock() {
        *guard = config.clone();
    }
}

pub fn current_config() -> Config {
    live_slot()
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| load_config().unwrap_or_default())
}
