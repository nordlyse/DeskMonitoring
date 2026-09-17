use std::f64::consts::PI;
use std::sync::OnceLock;
use std::time::Instant;

use gtk::cairo::Context;

use crate::snapshot::Snapshot;
use crate::theme::Palette;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sky {
    Sun,
    Clouds,
    Rain,
    Snow,
}

impl Sky {
    fn from_code(code: Option<i32>, is_day: bool) -> Option<Self> {
        match code? {
            0 | 1 if is_day => Some(Self::Sun),
            0 | 1 => None,
            2 | 3 | 45 | 48 => Some(Self::Clouds),
            51 | 53 | 55 | 56 | 57 | 61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 | 95 | 96 | 99 => {
                Some(Self::Rain)
            }
            71 | 73 | 75 | 77 | 85 | 86 => Some(Self::Snow),
            _ => None,
        }
    }
}

pub fn paint(cr: &Context, w: f64, h: f64, snapshot: &Snapshot, palette: Palette) {
    let Some(sky) = Sky::from_code(snapshot.weather_code, snapshot.weather_is_day) else {
        return;
    };
    let t = elapsed();
    match sky {
        Sky::Sun => paint_sun(cr, w, h, t, palette),
        Sky::Clouds => paint_clouds(cr, w, h, t, palette),
        Sky::Rain => paint_rain(cr, w, h, t, palette),
        Sky::Snow => paint_snow(cr, w, h, t, palette),
    }
}

fn paint_sun(cr: &Context, w: f64, h: f64, t: f64, palette: Palette) {
    let cx = w - 42.0;
    let cy = 42.0;
    let n = palette.neon;
    let reach = (w * w + h * h).sqrt();
    let spin = t * 0.22;
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    for i in 0..18 {
        let a = spin + i as f64 * PI / 9.0;
        let inner = 16.0 + (i % 3) as f64 * 2.0;
        cr.set_line_width(if i % 2 == 0 { 2.2 } else { 1.2 });
        cr.set_source_rgba(n.r, n.g, n.b, if i % 2 == 0 { 0.22 } else { 0.12 });
        cr.move_to(cx + a.cos() * inner, cy + a.sin() * inner);
        cr.line_to(cx + a.cos() * reach, cy + a.sin() * reach);
        cr.stroke().ok();
    }
    let pulse = 15.0 + (t * 1.7).sin() * 1.6;
    for k in (1..=5).rev() {
        let r = pulse + k as f64 * 6.5;
        cr.set_source_rgba(n.r, n.g, n.b, 0.16 / k as f64);
        cr.arc(cx, cy, r, 0.0, 2.0 * PI);
        cr.fill().ok();
    }
    cr.set_source_rgba(n.r, n.g, n.b, 0.88);
    cr.arc(cx, cy, pulse, 0.0, 2.0 * PI);
    cr.fill().ok();
}

fn paint_clouds(cr: &Context, w: f64, h: f64, t: f64, palette: Palette) {
    for i in 0..6u64 {
        let s = unit(i.wrapping_add(11));
        let scale = 0.72 + unit(i.wrapping_add(29)) * 0.55;
        let cloud_w = 118.0 * scale;
        let speed = 16.0 + unit(i.wrapping_add(41)) * 22.0;
        let span = w + cloud_w + 40.0;
        let x = ((t * speed + s * span) % span) - cloud_w * 0.55;
        let y = 36.0 + unit(i.wrapping_add(17)) * (h * 0.62);
        paint_cloud(cr, x, y, scale, palette);
    }
}

fn paint_cloud(cr: &Context, x: f64, y: f64, scale: f64, palette: Palette) {
    let n = palette.neon;
    let d = palette.dim;
    cr.set_source_rgba(d.r, d.g, d.b, 0.28);
    fill_puff(cr, x, y, 34.0 * scale, 16.0 * scale);
    fill_puff(cr, x + 28.0 * scale, y - 10.0 * scale, 30.0 * scale, 18.0 * scale);
    fill_puff(cr, x + 54.0 * scale, y + 2.0 * scale, 32.0 * scale, 15.0 * scale);
    fill_puff(cr, x + 22.0 * scale, y + 8.0 * scale, 36.0 * scale, 14.0 * scale);
    cr.set_source_rgba(n.r, n.g, n.b, 0.18);
    fill_puff(cr, x + 16.0 * scale, y - 4.0 * scale, 26.0 * scale, 12.0 * scale);
}

fn fill_puff(cr: &Context, cx: f64, cy: f64, rx: f64, ry: f64) {
    cr.save().ok();
    cr.translate(cx, cy);
    cr.scale(rx.max(1.0), ry.max(1.0));
    cr.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI);
    cr.fill().ok();
    cr.restore().ok();
}

fn paint_rain(cr: &Context, w: f64, h: f64, t: f64, palette: Palette) {
    let n = palette.neon;
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    for i in 0..72u64 {
        let x = 8.0 + unit(i) * (w - 16.0);
        let len = 10.0 + unit(i.wrapping_add(3)) * 18.0;
        let speed = 280.0 + unit(i.wrapping_add(7)) * 260.0;
        let cycle = h + len + 24.0;
        let y = ((t * speed + unit(i.wrapping_add(13)) * cycle) % cycle) - len;
        cr.set_line_width(1.0 + unit(i.wrapping_add(19)) * 1.1);
        cr.set_source_rgba(n.r, n.g, n.b, 0.28 + unit(i.wrapping_add(23)) * 0.42);
        cr.move_to(x, y);
        cr.line_to(x - 1.2, y + len);
        cr.stroke().ok();
    }
}

fn paint_snow(cr: &Context, w: f64, h: f64, t: f64, palette: Palette) {
    let n = palette.neon;
    cr.set_line_cap(gtk::cairo::LineCap::Round);
    for i in 0..56u64 {
        let base_x = 6.0 + unit(i) * (w - 12.0);
        let r = 1.4 + unit(i.wrapping_add(5)) * 2.4;
        let speed = 28.0 + unit(i.wrapping_add(9)) * 46.0;
        let cycle = h + r * 6.0 + 20.0;
        let y = ((t * speed + unit(i.wrapping_add(15)) * cycle) % cycle) - r * 3.0;
        let x = base_x + (t * (0.7 + unit(i.wrapping_add(21))) + i as f64).sin() * 14.0;
        let a = 0.32 + unit(i.wrapping_add(27)) * 0.48;
        cr.set_source_rgba(n.r, n.g, n.b, a);
        cr.arc(x, y, r * 0.45, 0.0, 2.0 * PI);
        cr.fill().ok();
        cr.set_line_width(0.9);
        let rot = t * 0.6 + unit(i.wrapping_add(31)) * PI;
        for arm in 0..6 {
            let ang = rot + arm as f64 * PI / 3.0;
            cr.move_to(x - ang.cos() * r, y - ang.sin() * r);
            cr.line_to(x + ang.cos() * r, y + ang.sin() * r);
            cr.stroke().ok();
        }
    }
}

fn elapsed() -> f64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn unit(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z ^= z >> 32;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 29;
    (z as f64) / (u64::MAX as f64)
}
