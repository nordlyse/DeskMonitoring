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
    pub panel: Rgba,
    pub grid: Rgba,
    pub rain: Rgba,
    pub warn: Rgba,
}

impl Palette {
    pub fn from_kind(kind: PaletteKind) -> Self {
        match kind {
            PaletteKind::Matrix => Self {
                kind,
                neon: Rgba::rgb(0.10, 1.00, 0.38),
                dim: Rgba::rgb(0.05, 0.42, 0.18),
                text: Rgba::rgb(0.78, 1.00, 0.84),
                muted: Rgba::rgb(0.35, 0.72, 0.48),
                panel: Rgba {
                    r: 0.01,
                    g: 0.06,
                    b: 0.03,
                    a: 0.38,
                },
                grid: Rgba {
                    r: 0.10,
                    g: 1.00,
                    b: 0.38,
                    a: 0.10,
                },
                rain: Rgba {
                    r: 0.10,
                    g: 1.00,
                    b: 0.38,
                    a: 0.28,
                },
                warn: Rgba::rgb(1.00, 0.82, 0.20),
            },
            PaletteKind::Turquoise => Self {
                kind,
                neon: Rgba::rgb(0.18, 0.95, 0.88),
                dim: Rgba::rgb(0.08, 0.38, 0.40),
                text: Rgba::rgb(0.82, 1.00, 0.98),
                muted: Rgba::rgb(0.40, 0.78, 0.76),
                panel: Rgba {
                    r: 0.01,
                    g: 0.07,
                    b: 0.08,
                    a: 0.38,
                },
                grid: Rgba {
                    r: 0.18,
                    g: 0.95,
                    b: 0.88,
                    a: 0.10,
                },
                rain: Rgba {
                    r: 0.18,
                    g: 0.95,
                    b: 0.88,
                    a: 0.28,
                },
                warn: Rgba::rgb(1.00, 0.78, 0.32),
            },
            PaletteKind::Blue => Self {
                kind,
                neon: Rgba::rgb(0.25, 0.62, 1.00),
                dim: Rgba::rgb(0.08, 0.22, 0.48),
                text: Rgba::rgb(0.82, 0.92, 1.00),
                muted: Rgba::rgb(0.45, 0.62, 0.90),
                panel: Rgba {
                    r: 0.02,
                    g: 0.04,
                    b: 0.10,
                    a: 0.38,
                },
                grid: Rgba {
                    r: 0.25,
                    g: 0.62,
                    b: 1.00,
                    a: 0.10,
                },
                rain: Rgba {
                    r: 0.25,
                    g: 0.62,
                    b: 1.00,
                    a: 0.28,
                },
                warn: Rgba::rgb(1.00, 0.72, 0.28),
            },
            PaletteKind::Pink => Self {
                kind,
                neon: Rgba::rgb(1.00, 0.38, 0.78),
                dim: Rgba::rgb(0.42, 0.10, 0.32),
                text: Rgba::rgb(1.00, 0.88, 0.96),
                muted: Rgba::rgb(0.86, 0.52, 0.74),
                panel: Rgba {
                    r: 0.08,
                    g: 0.02,
                    b: 0.06,
                    a: 0.38,
                },
                grid: Rgba {
                    r: 1.00,
                    g: 0.38,
                    b: 0.78,
                    a: 0.10,
                },
                rain: Rgba {
                    r: 1.00,
                    g: 0.38,
                    b: 0.78,
                    a: 0.28,
                },
                warn: Rgba::rgb(1.00, 0.86, 0.28),
            },
            PaletteKind::Yellow => Self {
                kind,
                neon: Rgba::rgb(1.00, 0.88, 0.18),
                dim: Rgba::rgb(0.42, 0.34, 0.04),
                text: Rgba::rgb(1.00, 0.97, 0.78),
                muted: Rgba::rgb(0.86, 0.78, 0.38),
                panel: Rgba {
                    r: 0.07,
                    g: 0.06,
                    b: 0.01,
                    a: 0.38,
                },
                grid: Rgba {
                    r: 1.00,
                    g: 0.88,
                    b: 0.18,
                    a: 0.10,
                },
                rain: Rgba {
                    r: 1.00,
                    g: 0.88,
                    b: 0.18,
                    a: 0.28,
                },
                warn: Rgba::rgb(1.00, 0.45, 0.22),
            },
        }
    }
}

pub fn window_css() -> &'static str {
    r#"
window.desk-monitor {
  background-color: transparent;
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
