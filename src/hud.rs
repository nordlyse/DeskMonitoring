use std::collections::VecDeque;
use std::f64::consts::PI;

use chrono::Local;
use gtk::cairo::Context;

use crate::config::{Config, Position};
use crate::metrics::{format_bps, format_bytes};
use crate::snapshot::Snapshot;
use crate::theme::{Palette, Rgba};

const SETTINGS_LABEL: &str = "SETTINGS";

pub fn paint(cr: &Context, width: i32, height: i32, snapshot: &Snapshot, config: &Config) {
    let w = width.max(1) as f64;
    let h = height.max(1) as f64;
    let palette = Palette::from_kind(config.palette);
    cr.set_operator(gtk::cairo::Operator::Source);
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    cr.paint().ok();
    cr.set_operator(gtk::cairo::Operator::Over);
    paint_wash(cr, w, h);
    rounded_rect(cr, 4.0, 4.0, w - 8.0, h - 8.0, 12.0);
    cr.clip();
    crate::weather_fx::paint(cr, w, h, snapshot, palette);
    crate::strands::paint(cr, w, h, palette);
    paint_vertical(cr, w, h, snapshot, config, palette);
}

pub fn hit_settings(x: f64, y: f64, width: i32, _position: Position) -> bool {
    let (pad, origin_y) = header_origin();
    let right = pad + (width.max(1) as f64 - pad * 2.0);
    let left = right - 108.0;
    let top = origin_y + 8.0;
    let bottom = origin_y + 36.0;
    x >= left && x <= right && y >= top && y <= bottom
}

fn header_origin() -> (f64, f64) {
    (22.0, 28.0)
}

fn paint_wash(cr: &Context, w: f64, h: f64) {
    rounded_rect(cr, 4.0, 4.0, w - 8.0, h - 8.0, 12.0);
    cr.set_source_rgba(0.06, 0.06, 0.07, 0.26);
    cr.fill().ok();
}

#[derive(Clone, Copy)]
enum Block {
    Cpu,
    Ram,
    Disk,
    Net,
    Mail,
    Cal,
    Wx,
}

impl Block {
    fn weight(self) -> f64 {
        match self {
            Self::Cpu | Self::Ram | Self::Net => 1.6,
            Self::Disk => 1.1,
            Self::Mail => 1.3,
            Self::Cal | Self::Wx => 1.2,
        }
    }

    fn from_config(config: &Config) -> Vec<Self> {
        let mut blocks = Vec::new();
        if config.panels.cpu {
            blocks.push(Self::Cpu);
        }
        if config.panels.ram {
            blocks.push(Self::Ram);
        }
        if config.panels.disk {
            blocks.push(Self::Disk);
        }
        if config.panels.network {
            blocks.push(Self::Net);
        }
        if config.panels.mail {
            blocks.push(Self::Mail);
        }
        if config.panels.calendar {
            blocks.push(Self::Cal);
        }
        if config.panels.weather {
            blocks.push(Self::Wx);
        }
        blocks
    }
}

fn paint_vertical(cr: &Context, w: f64, h: f64, snapshot: &Snapshot, config: &Config, palette: Palette) {
    let pad = 22.0;
    let inner_w = w - pad * 2.0;
    let mut y = 28.0;
    y += paint_header(cr, pad, y, inner_w, palette) + 10.0;
    let blocks = Block::from_config(config);
    if blocks.is_empty() {
        text(
            cr,
            pad,
            y + 24.0,
            13.0,
            "Open SETTINGS to add panels",
            palette.muted,
            false,
        );
        return;
    }
    let gap = 10.0;
    let leftover = (h - y - pad - gap * blocks.len().saturating_sub(1) as f64).max(80.0);
    let total: f64 = blocks.iter().map(|b| b.weight()).sum();
    for block in blocks {
        let bh = leftover * (block.weight() / total);
        paint_block(cr, pad, y, inner_w, bh, block, snapshot, palette, false);
        y += bh + gap;
    }
}

fn paint_block(
    cr: &Context,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    block: Block,
    snapshot: &Snapshot,
    palette: Palette,
    compact: bool,
) {
    match block {
        Block::Cpu => {
            paint_labeled_panel(cr, x, y, w, h, if compact { "CPU" } else { "CPU LOAD" }, palette);
            if compact {
                paint_gauge(cr, x + 18.0, y + 26.0, h - 48.0, snapshot.cpu_pct, palette);
            } else {
                paint_gauge(cr, x + 18.0, y + 28.0, 78.0, snapshot.cpu_pct, palette);
                paint_sparkline(cr, x + 110.0, y + 34.0, w - 128.0, h - 48.0, &snapshot.cpu_history, palette, false);
                text(cr, x + 110.0, y + 26.0, 13.0, &format!("{:.0}%", snapshot.cpu_pct), palette.neon, false);
            }
        }
        Block::Ram => {
            paint_labeled_panel(cr, x, y, w, h, if compact { "RAM" } else { "MEMORY" }, palette);
            if compact {
                paint_gauge(cr, x + 18.0, y + 26.0, h - 48.0, snapshot.ram_pct(), palette);
            } else {
                paint_gauge(cr, x + 18.0, y + 28.0, 78.0, snapshot.ram_pct(), palette);
                paint_sparkline(cr, x + 110.0, y + 34.0, w - 128.0, h - 48.0, &snapshot.ram_history, palette, false);
                text(
                    cr,
                    x + 110.0,
                    y + 26.0,
                    12.0,
                    &format!(
                        "{} / {}",
                        format_bytes(snapshot.ram_used),
                        format_bytes(snapshot.ram_total)
                    ),
                    palette.muted,
                    false,
                );
            }
        }
        Block::Disk => {
            paint_labeled_panel(cr, x, y, w, h, "DISK", palette);
            if compact {
                paint_mini_disk(cr, x + 10.0, y + 28.0, w - 20.0, h - 40.0, snapshot, palette);
            } else {
                paint_disk_bar(cr, x + 16.0, y + 32.0, w - 32.0, snapshot, palette);
            }
        }
        Block::Net => {
            paint_labeled_panel(cr, x, y, w, h, "NETWORK", palette);
            if compact {
                paint_dual_area(cr, x + 10.0, y + 28.0, w - 20.0, h - 40.0, &snapshot.net_down_history, &snapshot.net_up_history, palette);
            } else {
                text(
                    cr,
                    x + 16.0,
                    y + 28.0,
                    12.0,
                    &format!(
                        "DOWN {}   UP {}",
                        format_bps(snapshot.net_down_bps),
                        format_bps(snapshot.net_up_bps)
                    ),
                    palette.text,
                    false,
                );
                paint_dual_area(cr, x + 16.0, y + 40.0, w - 32.0, h - 52.0, &snapshot.net_down_history, &snapshot.net_up_history, palette);
            }
        }
        Block::Mail => {
            paint_labeled_panel(cr, x, y, w, h, "MAIL", palette);
            let inset = if compact { 8.0 } else { 12.0 };
            paint_mail_table(cr, x + inset, y + 26.0, w - inset * 2.0, h - 36.0, snapshot, palette);
        }
        Block::Cal => {
            paint_labeled_panel(cr, x, y, w, h, if compact { "CALENDAR" } else { "CALENDAR  TODAY" }, palette);
            paint_calendar(cr, x + 16.0, y + 30.0, w - 32.0, h - 40.0, snapshot, palette);
        }
        Block::Wx => {
            paint_labeled_panel(cr, x, y, w, h, "WEATHER", palette);
            paint_weather(cr, x + 16.0, y + 30.0, w - 32.0, snapshot, palette);
        }
    }
}

fn paint_header(cr: &Context, x: f64, y: f64, width: f64, palette: Palette) -> f64 {
    text(cr, x, y, 18.0, "DESK MONITOR", palette.neon, true);
    let clock = Local::now().format("%Y-%m-%d  %H:%M:%S").to_string();
    text_right(cr, x + width, y, 13.0, &clock, palette.muted);
    text(
        cr,
        x,
        y + 20.0,
        11.0,
        palette.kind.title(),
        palette.muted,
        false,
    );
    text_right(cr, x + width, y + 20.0, 11.0, SETTINGS_LABEL, palette.neon);
    36.0
}

fn paint_labeled_panel(cr: &Context, x: f64, y: f64, _w: f64, _h: f64, title: &str, palette: Palette) {
    text(cr, x + 12.0, y + 14.0, 11.0, title, palette.neon, true);
}

fn paint_gauge(cr: &Context, x: f64, y: f64, size: f64, pct: f32, palette: Palette) {
    let cx = x + size / 2.0;
    let cy = y + size / 2.0;
    let r = (size / 2.0 - 6.0).max(18.0);
    cr.set_line_width(8.0);
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.85);
    cr.new_path();
    cr.arc(cx, cy, r, 0.75 * PI, 2.25 * PI);
    cr.stroke().ok();

    let t = (pct as f64 / 100.0).clamp(0.0, 1.0);
    let end = 0.75 * PI + t * 1.5 * PI;
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 1.0);
    cr.new_path();
    cr.arc(cx, cy, r, 0.75 * PI, end);
    cr.stroke().ok();
    cr.set_line_cap(gtk::cairo::LineCap::Butt);
    text_center(cr, cx, cy + 5.0, 14.0, &format!("{:.0}%", pct), palette.neon);
}

fn paint_sparkline(
    cr: &Context,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    data: &VecDeque<f32>,
    palette: Palette,
    fill_area: bool,
) {
    if data.len() < 2 || w <= 1.0 || h <= 1.0 {
        return;
    }
    path_series(cr, x, y, w, h, data, 100.0);
    if fill_area {
        cr.line_to(x + w, y + h);
        cr.line_to(x, y + h);
        cr.close_path();
        cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.28);
        cr.fill().ok();
        path_series(cr, x, y, w, h, data, 100.0);
    }
    cr.set_line_width(2.4);
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 1.0);
    cr.stroke().ok();
}

fn paint_dual_area(
    cr: &Context,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    down: &VecDeque<f32>,
    up: &VecDeque<f32>,
    palette: Palette,
) {
    let max = down
        .iter()
        .chain(up.iter())
        .copied()
        .fold(1.0f32, f32::max)
        .max(1.0);
    path_series(cr, x, y, w, h, down, max);
    cr.line_to(x + w, y + h);
    cr.line_to(x, y + h);
    cr.close_path();
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.28);
    cr.fill().ok();
    path_series(cr, x, y, w, h, down, max);
    cr.set_line_width(2.4);
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 1.0);
    cr.stroke().ok();
    path_series(cr, x, y, w, h, up, max);
    cr.set_line_width(2.2);
    cr.set_source_rgba(palette.warn.r, palette.warn.g, palette.warn.b, 1.0);
    cr.stroke().ok();
}

fn path_series(cr: &Context, x: f64, y: f64, w: f64, h: f64, data: &VecDeque<f32>, max: f32) {
    let n = data.len().max(1) as f64;
    cr.new_path();
    for (i, value) in data.iter().enumerate() {
        let px = x + (i as f64 / (n - 1.0).max(1.0)) * w;
        let t = (*value / max).clamp(0.0, 1.0) as f64;
        let py = y + h - t * h;
        if i == 0 {
            cr.move_to(px, py);
        } else {
            cr.line_to(px, py);
        }
    }
}

fn paint_disk_bar(cr: &Context, x: f64, y: f64, w: f64, snapshot: &Snapshot, palette: Palette) {
    let used_pct = snapshot.disk_used_pct() as f64 / 100.0;
    cr.set_line_width(8.0);
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.9);
    cr.move_to(x, y + 9.0);
    cr.line_to(x + w, y + 9.0);
    cr.stroke().ok();
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 1.0);
    cr.move_to(x, y + 9.0);
    cr.line_to(x + (w * used_pct).max(2.0), y + 9.0);
    cr.stroke().ok();
    cr.set_line_cap(gtk::cairo::LineCap::Butt);
    text(
        cr,
        x,
        y + 36.0,
        12.0,
        &format!(
            "Used {}   Free {}   Total {}",
            format_bytes(snapshot.disk_used),
            format_bytes(snapshot.disk_free),
            format_bytes(snapshot.disk_total)
        ),
        palette.text,
        false,
    );
}

fn paint_mini_disk(cr: &Context, x: f64, y: f64, w: f64, h: f64, snapshot: &Snapshot, palette: Palette) {
    paint_pie(cr, x + w / 2.0, y + h * 0.38, (h * 0.28).min(w * 0.32), snapshot.disk_used_pct(), palette);
    text_center(
        cr,
        x + w / 2.0,
        y + h * 0.78,
        11.0,
        &format!("Free {}", format_bytes(snapshot.disk_free)),
        palette.text,
    );
}

fn paint_pie(cr: &Context, cx: f64, cy: f64, r: f64, used_pct: f32, palette: Palette) {
    let used = (used_pct as f64 / 100.0).clamp(0.0, 1.0) * 2.0 * PI;
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.9);
    cr.new_path();
    cr.move_to(cx, cy);
    cr.arc(cx, cy, r, 0.0, 2.0 * PI);
    cr.close_path();
    cr.fill().ok();
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 1.0);
    cr.new_path();
    cr.move_to(cx, cy);
    cr.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + used);
    cr.close_path();
    cr.fill().ok();
}

fn paint_mail_table(
    cr: &Context,
    x: f64,
    y: f64,
    w: f64,
    _h: f64,
    snapshot: &Snapshot,
    palette: Palette,
) {
    let labels = ["IN", "OUT", "TOTAL", "UNREAD"];
    let values = [
        snapshot.mail_in,
        snapshot.mail_out,
        snapshot.mail_total,
        snapshot.mail_unread,
    ];
    let cell_w = w / 4.0;
    for i in 0..4 {
        let cx = x + i as f64 * cell_w;
        text_center(cr, cx + cell_w / 2.0, y + 16.0, 10.0, labels[i], palette.muted);
        text_center(
            cr,
            cx + cell_w / 2.0,
            y + 40.0,
            18.0,
            &values[i].to_string(),
            palette.neon,
        );
    }
}

fn paint_calendar(
    cr: &Context,
    x: f64,
    y: f64,
    _w: f64,
    h: f64,
    snapshot: &Snapshot,
    palette: Palette,
) {
    if snapshot.calendar_events.is_empty() {
        text(cr, x, y + 18.0, 28.0, "0", palette.neon, true);
        text(cr, x + 36.0, y + 18.0, 13.0, "no events today", palette.muted, false);
        return;
    }
    text(
        cr,
        x,
        y,
        12.0,
        &format!("{} event(s)", snapshot.calendar_events.len()),
        palette.text,
        false,
    );
    let mut yy = y + 18.0;
    for line in snapshot.calendar_events.iter().take(((h - 20.0) / 16.0) as usize) {
        text(cr, x, yy, 12.0, line, palette.muted, false);
        yy += 16.0;
    }
}

fn paint_weather(cr: &Context, x: f64, y: f64, _w: f64, snapshot: &Snapshot, palette: Palette) {
    text(cr, x, y, 14.0, &snapshot.weather_city, palette.text, true);
    let temp = snapshot
        .weather_temp_c
        .map(|t| format!("{t:.0}°C"))
        .unwrap_or_else(|| "--".to_string());
    text(cr, x, y + 28.0, 26.0, &temp, palette.neon, true);
    text(cr, x + 90.0, y + 28.0, 13.0, &snapshot.weather_desc, palette.muted, false);
    let mut meta = Vec::new();
    if let Some(h) = snapshot.weather_humidity {
        meta.push(format!("Humidity {h:.0}%"));
    }
    if let Some(wind) = snapshot.weather_wind {
        meta.push(format!("Wind {wind:.0} km/h"));
    }
    if !meta.is_empty() {
        text(cr, x, y + 50.0, 12.0, &meta.join("   "), palette.muted, false);
    }
}

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_path();
    cr.move_to(x + r, y);
    cr.line_to(x + w - r, y);
    cr.curve_to(x + w, y, x + w, y, x + w, y + r);
    cr.line_to(x + w, y + h - r);
    cr.curve_to(x + w, y + h, x + w, y + h, x + w - r, y + h);
    cr.line_to(x + r, y + h);
    cr.curve_to(x, y + h, x, y + h, x, y + h - r);
    cr.line_to(x, y + r);
    cr.curve_to(x, y, x, y, x + r, y);
    cr.close_path();
}

fn text(cr: &Context, x: f64, y: f64, size: f64, value: &str, color: Rgba, bold: bool) {
    let weight = if bold {
        gtk::cairo::FontWeight::Bold
    } else {
        gtk::cairo::FontWeight::Normal
    };
    cr.select_font_face("monospace", gtk::cairo::FontSlant::Normal, weight);
    cr.set_font_size(size);
    cr.set_source_rgba(color.r, color.g, color.b, color.a);
    cr.move_to(x, y);
    cr.show_text(value).ok();
}

fn text_center(cr: &Context, cx: f64, cy: f64, size: f64, value: &str, color: Rgba) {
    cr.select_font_face(
        "monospace",
        gtk::cairo::FontSlant::Normal,
        gtk::cairo::FontWeight::Bold,
    );
    cr.set_font_size(size);
    if let Ok(ext) = cr.text_extents(value) {
        cr.set_source_rgba(color.r, color.g, color.b, color.a);
        cr.move_to(cx - ext.width() / 2.0 - ext.x_bearing(), cy);
        cr.show_text(value).ok();
    }
}

fn text_right(cr: &Context, right: f64, y: f64, size: f64, value: &str, color: Rgba) {
    cr.select_font_face(
        "monospace",
        gtk::cairo::FontSlant::Normal,
        gtk::cairo::FontWeight::Normal,
    );
    cr.set_font_size(size);
    if let Ok(ext) = cr.text_extents(value) {
        cr.set_source_rgba(color.r, color.g, color.b, color.a);
        cr.move_to(right - ext.width() - ext.x_bearing(), y);
        cr.show_text(value).ok();
    }
}
