use crate::config::PaletteKind;

#[derive(Clone, Copy, Debug)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Rgba {
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub kind: PaletteKind,
    pub neon: Rgba,
    pub dim: Rgba,
    pub text: Rgba,
    pub muted: Rgba,
    pub warn: Rgba,
}

impl Palette {
    pub fn from_kind(kind: PaletteKind) -> Self {
        match kind {
            PaletteKind::Matrix => Self {
                kind,
                neon: Rgba::rgb(0.00, 0.78, 0.22),
                dim: Rgba::rgb(0.02, 0.28, 0.10),
                text: Rgba::rgb(0.22, 0.78, 0.38),
                muted: Rgba::rgb(0.08, 0.48, 0.22),
                warn: Rgba::rgb(0.88, 0.62, 0.04),
            },
            PaletteKind::Turquoise => Self {
                kind,
                neon: Rgba::rgb(0.00, 0.68, 0.66),
                dim: Rgba::rgb(0.00, 0.26, 0.28),
                text: Rgba::rgb(0.04, 0.64, 0.62),
                muted: Rgba::rgb(0.02, 0.44, 0.44),
                warn: Rgba::rgb(0.86, 0.54, 0.06),
            },
            PaletteKind::Blue => Self {
                kind,
                neon: Rgba::rgb(0.04, 0.38, 0.86),
                dim: Rgba::rgb(0.02, 0.14, 0.36),
                text: Rgba::rgb(0.18, 0.48, 0.86),
                muted: Rgba::rgb(0.08, 0.30, 0.62),
                warn: Rgba::rgb(0.88, 0.52, 0.06),
            },
            PaletteKind::Pink => Self {
                kind,
                neon: Rgba::rgb(0.86, 0.12, 0.48),
                dim: Rgba::rgb(0.32, 0.02, 0.16),
                text: Rgba::rgb(0.86, 0.28, 0.52),
                muted: Rgba::rgb(0.62, 0.10, 0.34),
                warn: Rgba::rgb(0.88, 0.62, 0.06),
            },
            PaletteKind::Yellow => Self {
                kind,
                neon: Rgba::rgb(0.86, 0.64, 0.00),
                dim: Rgba::rgb(0.32, 0.22, 0.00),
                text: Rgba::rgb(0.82, 0.66, 0.08),
                muted: Rgba::rgb(0.58, 0.44, 0.02),
                warn: Rgba::rgb(0.86, 0.32, 0.04),
            },
        }
    }
}

pub fn window_css() -> &'static str {
    r#"
window.desk-monitor,
window.desk-monitor.csd,
window.desk-monitor.background,
window.desk-monitor box,
window.desk-monitor drawingarea,
window.desk-monitor .desk-monitor-host,
window.desk-monitor .desk-monitor-canvas {
  background: none;
  background-color: transparent;
  box-shadow: none;
  border: none;
  border-radius: 0;
  outline: none;
  margin: 0;
  padding: 0;
}
window.desk-setup {
  background-color: #101418;
  color: #ffffff;
}
window.desk-setup label,
window.desk-setup checkbutton,
window.desk-setup checkbutton label,
window.desk-setup button,
window.desk-setup button label,
window.desk-setup entry,
window.desk-setup textview,
window.desk-setup text {
  color: #ffffff;
}
window.desk-setup .title-1,
window.desk-setup .heading,
window.desk-setup .error {
  color: #ffffff;
}
window.desk-setup entry {
  background-color: #1c2228;
  caret-color: #ffffff;
}
window.desk-setup entry placeholder {
  color: #d0d0d0;
}
window.desk-setup button {
  background-color: #2a333c;
  color: #ffffff;
}
window.desk-setup button.suggested-action {
  background-color: #1f6feb;
  color: #ffffff;
}
window.desk-setup scrolledwindow,
window.desk-setup viewport,
window.desk-setup box {
  background-color: #101418;
  color: #ffffff;
}
"#
}
