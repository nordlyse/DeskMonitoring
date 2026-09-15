use std::collections::VecDeque;
use std::f64::consts::PI;

use chrono::Local;
use gtk::cairo::{Context, LinearGradient};

use crate::config::Position;
use crate::metrics::{format_bps, format_bytes};
use crate::snapshot::Snapshot;
use crate::theme::{Palette, Rgba};

const GLYPHS: &[u8] = b"01ABCDEFGHKMNPQRSTUVWXYZ#$%+*<>";

pub struct RainColumn {
    x: f64,
    y: f64,
    speed: f64,
    glyphs: Vec<char>,
}

pub struct FxState {
    rng: u64,
    columns: Vec<RainColumn>,
    last_w: i32,
    last_h: i32,
}

impl FxState {
    pub fn add() -> Self {
        Self {
            rng: 0x9E37_79B9_7F4A_7C15,
            columns: Vec::new(),
            last_w: 0,
            last_h: 0,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }

    pub fn step(&mut self, width: i32, height: i32) {
        if width != self.last_w || height != self.last_h || self.columns.is_empty() {
            self.last_w = width;
            self.last_h = height;
            self.rebuild(width, height);
        }
        let h = height as f64;
        let mut rng = self.rng;
        for column in &mut self.columns {
            column.y += column.speed;
            if column.y > h + 80.0 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                column.y = -120.0 - (rng as f64 / u64::MAX as f64) * 200.0;
            }
        }
        self.rng = rng;
    }

    fn rebuild(&mut self, width: i32, height: i32) {
        let count = ((width / 18).clamp(8, 42)) as usize;
        self.columns.clear();
        for i in 0..count {
            let len = 8 + (self.next_u64() as usize % 12);
            let mut glyphs = Vec::with_capacity(len);
            for _ in 0..len {
                glyphs.push(glyph(self.next_u64()));
            }
            let y = self.next_f64() * height as f64;
            let speed = 1.6 + self.next_f64() * 4.2;
            self.columns.push(RainColumn {
                x: (i as f64 + 0.35) * (width as f64 / count as f64),
                y,
                speed,
                glyphs,
            });
        }
    }
}

fn glyph(seed: u64) -> char {
    GLYPHS[(seed as usize) % GLYPHS.len()] as char
}

pub fn paint(
    cr: &Context,
    width: i32,
    height: i32,
    snapshot: &Snapshot,
    palette: Palette,
    position: Position,
    fx: &FxState,
) {
    let w = width.max(1) as f64;
    let h = height.max(1) as f64;
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    cr.paint().ok();

    rounded_rect(cr, 8.0, 8.0, w - 16.0, h - 16.0, 14.0);
    fill(cr, palette.panel);
    stroke_neon(cr, palette.neon, 1.2, 0.55);

    paint_grid(cr, w, h, palette);
    paint_rain(cr, fx, palette);
    paint_scanlines(cr, w, h);

    if position.is_horizontal() {
        paint_horizontal(cr, w, h, snapshot, palette);
    } else {
        paint_vertical(cr, w, h, snapshot, palette);
    }
}

fn paint_vertical(cr: &Context, w: f64, h: f64, snapshot: &Snapshot, palette: Palette) {
    let pad = 22.0;
    let inner_w = w - pad * 2.0;
    let mut y = 28.0;
    y += paint_header(cr, pad, y, inner_w, snapshot, palette) + 10.0;

    let gauge_h = (h * 0.16).clamp(96.0, 130.0);
    paint_labeled_panel(cr, pad, y, inner_w, gauge_h, "CPU LOAD", palette);
    paint_gauge(cr, pad + 18.0, y + 28.0, 78.0, snapshot.cpu_pct, palette);
    paint_sparkline(
        cr,
        pad + 110.0,
        y + 34.0,
        inner_w - 128.0,
        gauge_h - 48.0,
        &snapshot.cpu_history,
        palette,
        false,
    );
    text(
        cr,
        pad + 110.0,
        y + 26.0,
        13.0,
        &format!("{:.0}%", snapshot.cpu_pct),
        palette.text,
        false,
    );
    y += gauge_h + 10.0;

    paint_labeled_panel(cr, pad, y, inner_w, gauge_h, "MEMORY", palette);
    paint_gauge(cr, pad + 18.0, y + 28.0, 78.0, snapshot.ram_pct(), palette);
    paint_sparkline(
        cr,
        pad + 110.0,
        y + 34.0,
        inner_w - 128.0,
        gauge_h - 48.0,
        &snapshot.ram_history,
        palette,
        false,
    );
    text(
        cr,
        pad + 110.0,
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
    y += gauge_h + 10.0;

    let disk_h = (h * 0.11).clamp(78.0, 100.0);
    paint_labeled_panel(cr, pad, y, inner_w, disk_h, "DISK", palette);
    paint_disk_bar(cr, pad + 16.0, y + 32.0, inner_w - 32.0, snapshot, palette);
    y += disk_h + 10.0;

    let net_h = (h * 0.16).clamp(100.0, 140.0);
    paint_labeled_panel(cr, pad, y, inner_w, net_h, "NETWORK", palette);
    text(
        cr,
        pad + 16.0,
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
    paint_dual_area(
        cr,
        pad + 16.0,
        y + 40.0,
        inner_w - 32.0,
        net_h - 52.0,
        &snapshot.net_down_history,
        &snapshot.net_up_history,
        palette,
    );
    y += net_h + 10.0;

    let mail_h = (h * 0.13).clamp(90.0, 120.0);
    paint_labeled_panel(cr, pad, y, inner_w, mail_h, "MAIL", palette);
    paint_mail_table(cr, pad + 12.0, y + 28.0, inner_w - 24.0, mail_h - 40.0, snapshot, palette);
    y += mail_h + 10.0;

    let cal_h = (h * 0.12).clamp(84.0, 120.0);
    paint_labeled_panel(cr, pad, y, inner_w, cal_h, "CALENDAR  TODAY", palette);
    paint_calendar(cr, pad + 16.0, y + 30.0, inner_w - 32.0, cal_h - 40.0, snapshot, palette);
    y += cal_h + 10.0;

    let wx_h = (h - y - pad).max(80.0);
    paint_labeled_panel(cr, pad, y, inner_w, wx_h, "WEATHER", palette);
    paint_weather(cr, pad + 16.0, y + 30.0, inner_w - 32.0, snapshot, palette);
}

fn paint_horizontal(cr: &Context, w: f64, h: f64, snapshot: &Snapshot, palette: Palette) {
    let pad = 16.0;
    let header_h = paint_header(cr, pad, 18.0, w - pad * 2.0, snapshot, palette);
    let y = 18.0 + header_h + 8.0;
    let body_h = h - y - pad;
    let gap = 8.0;
    let cols = 7.0;
    let col_w = (w - pad * 2.0 - gap * (cols - 1.0)) / cols;
    let mut x = pad;

    paint_labeled_panel(cr, x, y, col_w, body_h, "CPU", palette);
    paint_gauge(cr, x + 18.0, y + 26.0, body_h - 48.0, snapshot.cpu_pct, palette);
    x += col_w + gap;

    paint_labeled_panel(cr, x, y, col_w, body_h, "RAM", palette);
    paint_gauge(cr, x + 18.0, y + 26.0, body_h - 48.0, snapshot.ram_pct(), palette);
    x += col_w + gap;

    paint_labeled_panel(cr, x, y, col_w, body_h, "DISK", palette);
    paint_mini_disk(cr, x + 10.0, y + 28.0, col_w - 20.0, body_h - 40.0, snapshot, palette);
    x += col_w + gap;

    paint_labeled_panel(cr, x, y, col_w * 1.35, body_h, "NETWORK", palette);
    paint_dual_area(
        cr,
        x + 10.0,
        y + 28.0,
        col_w * 1.35 - 20.0,
        body_h - 40.0,
        &snapshot.net_down_history,
        &snapshot.net_up_history,
        palette,
    );
    x += col_w * 1.35 + gap;

    paint_labeled_panel(cr, x, y, col_w * 1.15, body_h, "MAIL", palette);
    paint_mail_table(cr, x + 8.0, y + 26.0, col_w * 1.15 - 16.0, body_h - 36.0, snapshot, palette);
    x += col_w * 1.15 + gap;

    let rest = w - pad - x;
    paint_labeled_panel(cr, x, y, rest, body_h * 0.48, "CALENDAR", palette);
    paint_calendar(cr, x + 10.0, y + 26.0, rest - 20.0, body_h * 0.48 - 34.0, snapshot, palette);
    paint_labeled_panel(
        cr,
        x,
        y + body_h * 0.48 + 8.0,
        rest,
        body_h * 0.52 - 8.0,
        "WEATHER",
        palette,
    );
    paint_weather(
        cr,
        x + 10.0,
        y + body_h * 0.48 + 34.0,
        rest - 20.0,
        snapshot,
        palette,
    );
}

fn paint_header(
    cr: &Context,
    x: f64,
    y: f64,
    width: f64,
    _snapshot: &Snapshot,
    palette: Palette,
) -> f64 {
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
    36.0
}

fn paint_labeled_panel(cr: &Context, x: f64, y: f64, w: f64, h: f64, title: &str, palette: Palette) {
    rounded_rect(cr, x, y, w, h, 10.0);
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.22);
    cr.fill_preserve().ok();
    stroke_neon(cr, palette.neon, 0.8, 0.28);
    text(cr, x + 12.0, y + 14.0, 11.0, title, palette.neon, true);
}

fn paint_gauge(cr: &Context, x: f64, y: f64, size: f64, pct: f32, palette: Palette) {
    let cx = x + size / 2.0;
    let cy = y + size / 2.0;
    let r = (size / 2.0 - 6.0).max(18.0);
    cr.set_line_width(8.0);
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.55);
    cr.arc(cx, cy, r, 0.75 * PI, 2.25 * PI);
    cr.stroke().ok();

    let t = (pct as f64 / 100.0).clamp(0.0, 1.0);
    let end = 0.75 * PI + t * 1.5 * PI;
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.95);
    cr.arc(cx, cy, r, 0.75 * PI, end);
    cr.stroke().ok();
    text_center(cr, cx, cy + 5.0, 14.0, &format!("{:.0}%", pct), palette.text);
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
    rounded_rect(cr, x, y, w, h, 6.0);
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
    cr.fill().ok();
    path_series(cr, x, y, w, h, data, 100.0);
    if fill_area {
        cr.line_to(x + w, y + h);
        cr.line_to(x, y + h);
        cr.close_path();
        cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.16);
        cr.fill().ok();
        path_series(cr, x, y, w, h, data, 100.0);
    }
    cr.set_line_width(1.6);
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.95);
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
    rounded_rect(cr, x, y, w, h, 6.0);
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
    cr.fill().ok();
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
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.18);
    cr.fill().ok();
    path_series(cr, x, y, w, h, down, max);
    cr.set_line_width(1.5);
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.95);
    cr.stroke().ok();
    path_series(cr, x, y, w, h, up, max);
    cr.set_line_width(1.4);
    cr.set_source_rgba(palette.warn.r, palette.warn.g, palette.warn.b, 0.9);
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
    rounded_rect(cr, x, y, w, 18.0, 6.0);
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.45);
    cr.fill().ok();
    rounded_rect(cr, x, y, (w * used_pct).max(2.0), 18.0, 6.0);
    let grad = LinearGradient::new(x, y, x + w, y);
    grad.add_color_stop_rgba(0.0, palette.neon.r, palette.neon.g, palette.neon.b, 0.55);
    grad.add_color_stop_rgba(1.0, palette.neon.r, palette.neon.g, palette.neon.b, 1.0);
    cr.set_source(&grad).ok();
    cr.fill().ok();
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
    cr.set_source_rgba(palette.dim.r, palette.dim.g, palette.dim.b, 0.5);
    cr.move_to(cx, cy);
    cr.arc(cx, cy, r, 0.0, 2.0 * PI);
    cr.fill().ok();
    cr.set_source_rgba(palette.neon.r, palette.neon.g, palette.neon.b, 0.95);
    cr.move_to(cx, cy);
    cr.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + used);
    cr.fill().ok();
}

fn paint_mail_table(
    cr: &Context,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
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
    let cell_h = h.max(48.0);
    for i in 0..4 {
        let cx = x + i as f64 * cell_w;
        rounded_rect(cr, cx + 4.0, y, cell_w - 8.0, cell_h, 8.0);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.16);
        cr.fill_preserve().ok();
        stroke_neon(cr, palette.neon, 0.7, 0.25);
        text_center(cr, cx + cell_w / 2.0, y + 16.0, 10.0, labels[i], palette.muted);
        text_center(
            cr,
            cx + cell_w / 2.0,
            y + 40.0,
            18.0,
            &values[i].to_string(),
            palette.text,
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

fn paint_grid(cr: &Context, w: f64, h: f64, palette: Palette) {
    cr.set_line_width(1.0);
    cr.set_source_rgba(palette.grid.r, palette.grid.g, palette.grid.b, palette.grid.a);
    let mut x = 20.0;
    while x < w {
        cr.move_to(x, 12.0);
        cr.line_to(x, h - 12.0);
        x += 28.0;
    }
    let mut y = 20.0;
    while y < h {
        cr.move_to(12.0, y);
        cr.line_to(w - 12.0, y);
        y += 28.0;
    }
    cr.stroke().ok();
}

fn paint_rain(cr: &Context, fx: &FxState, palette: Palette) {
    cr.select_font_face("monospace", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal);
    cr.set_font_size(12.0);
    for column in &fx.columns {
        for (i, ch) in column.glyphs.iter().enumerate() {
            let yy = column.y - i as f64 * 14.0;
            if yy < 10.0 {
                continue;
            }
            let alpha = if i == 0 { 0.9 } else { palette.rain.a * (1.0 - i as f64 / column.glyphs.len() as f64) };
            cr.set_source_rgba(palette.rain.r, palette.rain.g, palette.rain.b, alpha);
            cr.move_to(column.x, yy);
            cr.show_text(&ch.to_string()).ok();
        }
    }
}

fn paint_scanlines(cr: &Context, w: f64, h: f64) {
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
    cr.set_line_width(1.0);
    let mut y = 10.0;
    while y < h {
        cr.move_to(10.0, y);
        cr.line_to(w - 10.0, y);
        y += 3.0;
    }
    cr.stroke().ok();
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

fn fill(cr: &Context, color: Rgba) {
    cr.set_source_rgba(color.r, color.g, color.b, color.a);
    cr.fill_preserve().ok();
}

fn stroke_neon(cr: &Context, color: Rgba, width: f64, alpha: f64) {
    cr.set_line_width(width);
    cr.set_source_rgba(color.r, color.g, color.b, alpha);
    cr.stroke().ok();
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
