use std::f64::consts::PI;
use std::sync::OnceLock;
use std::time::Instant;

use gtk::cairo::{Context, LinearGradient};

use crate::theme::{Palette, Rgba};

const STRAND_COUNT: usize = 5;
const SPEED: f64 = 0.22;
const AMPLITUDE: f64 = 1.2;
const WAVINESS: f64 = 1.12;
const TAPER: f64 = 3.0;
const SPREAD: f64 = 1.0;
const SCALE: f64 = 1.35;
const INTENSITY: f64 = 0.68;

const SAMPLE: [Rgba; 4] = [
    Rgba::rgb(1.00, 0.26, 0.26),
    Rgba::rgb(0.49, 0.23, 0.93),
    Rgba::rgb(0.02, 0.71, 0.83),
    Rgba::rgb(0.92, 0.70, 0.03),
];

pub fn paint(cr: &Context, w: f64, h: f64, palette: Palette) {
    let t = elapsed();
    for i in 0..STRAND_COUNT {
        let pts = samples(w, h, t, i);
        if pts.len() < 3 {
            continue;
        }
        let base = i as f64 / STRAND_COUNT as f64 + t * 0.035;
        paint_ribbon(cr, &pts, 65.0, &stops(palette.neon, base), 0.04);
        paint_ribbon(cr, &pts, 28.0, &stops(palette.neon, base), 0.09);
        paint_ribbon(cr, &pts, 10.5, &stops(palette.neon, base), 0.28);
    }
}

fn stops(neon: Rgba, base: f64) -> [(f64, Rgba); 5] {
    [
        (0.00, sample(neon, base)),
        (0.22, sample(neon, base + 0.18)),
        (0.50, mix(sample(neon, base + 0.32), neon, 0.55)),
        (0.78, sample(neon, base + 0.52)),
        (1.00, sample(neon, base + 0.72)),
    ]
}

fn sample(neon: Rgba, t: f64) -> Rgba {
    let t = t.rem_euclid(1.0);
    let n = SAMPLE.len() as f64;
    let scaled = t * n;
    let idx = scaled.floor() as usize % SAMPLE.len();
    let next = (idx + 1) % SAMPLE.len();
    let accent = mix(SAMPLE[idx], SAMPLE[next], scaled.fract());
    mix(accent, neon, 0.34)
}

fn samples(w: f64, h: f64, t: f64, index: usize) -> Vec<(f64, f64, f64)> {
    let fi = index as f64;
    let ph = fi * 1.7 * SPREAD;
    let freq = (2.0 + fi * 0.35) * WAVINESS;
    let spd = 0.52 + fi * 0.16;
    let tt = t * SPEED;
    let e = 0.06 + INTENSITY * 0.94;
    let step = 4.0;
    let n = ((w / step).ceil() as usize).max(8);
    let mut out = Vec::with_capacity(n + 1);
    for k in 0..=n {
        let x = k as f64 * step;
        let ux = (x - 0.5 * w) / h / SCALE;
        let env = (ux * PI * 1.3).cos().max(0.0).powf(TAPER);
        if env <= 0.002 {
            continue;
        }
        let wave = (ux * freq + tt * spd + ph).sin() * 0.60
            + (ux * freq * 1.1 - tt * spd * 0.7 + ph * 1.7).sin() * 0.40;
        let amp = (0.1 + 0.02 * e) * env * AMPLITUDE;
        let uy = wave * amp;
        let y = 0.5 * h + uy * h * SCALE;
        out.push((x.clamp(0.0, w), y, env));
    }
    out
}

fn paint_ribbon(cr: &Context, pts: &[(f64, f64, f64)], half: f64, stops: &[(f64, Rgba)], alpha: f64) {
    let mut left = Vec::with_capacity(pts.len());
    let mut right = Vec::with_capacity(pts.len());
    for i in 0..pts.len() {
        let (x, y, env) = pts[i];
        let (x0, y0, _) = if i == 0 { pts[i] } else { pts[i - 1] };
        let (x1, y1, _) = if i + 1 == pts.len() {
            pts[i]
        } else {
            pts[i + 1]
        };
        let mut tx = x1 - x0;
        let mut ty = y1 - y0;
        let len = (tx * tx + ty * ty).sqrt().max(0.0001);
        tx /= len;
        ty /= len;
        let hw = half * (0.22 + env * 0.55);
        left.push((x - ty * hw, y + tx * hw));
        right.push((x + ty * hw, y - tx * hw));
    }
    cr.new_path();
    if let Some((x, y)) = left.first() {
        cr.move_to(*x, *y);
    }
    for (x, y) in left.iter().skip(1) {
        cr.line_to(*x, *y);
    }
    for (x, y) in right.iter().rev() {
        cr.line_to(*x, *y);
    }
    cr.close_path();
    let x0 = pts.first().map(|p| p.0).unwrap_or(0.0);
    let x1 = pts.last().map(|p| p.0).unwrap_or(x0 + 1.0);
    let y_mid = pts[pts.len() / 2].1;
    let gradient = LinearGradient::new(x0, y_mid, x1, y_mid);
    for (off, color) in stops {
        gradient.add_color_stop_rgba(*off, color.r, color.g, color.b, alpha);
    }
    cr.set_source(&gradient).ok();
    cr.fill().ok();
}

fn mix(a: Rgba, b: Rgba, t: f64) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    }
}

fn elapsed() -> f64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64()
}
