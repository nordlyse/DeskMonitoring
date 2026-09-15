use gtk::gdk::{self, Display, prelude::SurfaceExt};
use gtk::prelude::*;
use gtk::ApplicationWindow;

use crate::config::{Config, Position};

pub fn overlay_size(position: Position) -> (i32, i32) {
    match position {
        Position::Left | Position::Right => (380, 980),
        Position::Top | Position::Bottom => (1280, 280),
        Position::Center => (740, 900),
    }
}

pub fn apply_window_chrome(window: &ApplicationWindow) {
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_title(Some("Desk Monitor"));
    window.add_css_class("desk-monitor");
    window.connect_realize(|window| {
        if let Some(surface) = window.surface() {
            surface.set_opaque_region(None);
        }
    });
}

pub fn place_overlay(window: &ApplicationWindow, config: &Config) {
    let (mut width, mut height) = overlay_size(config.position);
    if let Some((mw, mh)) = monitor_size() {
        width = width.min((mw as f64 * 0.92) as i32).max(320);
        height = height.min((mh as f64 * 0.92) as i32).max(220);
    }
    window.set_default_size(width, height);

    #[cfg(all(feature = "layer-shell", target_os = "linux"))]
    {
        place_with_layer_shell(window, config.position, width, height);
    }
    #[cfg(not(all(feature = "layer-shell", target_os = "linux")))]
    {
        let _ = config;
        let _ = width;
        let _ = height;
    }
}

#[cfg(all(feature = "layer-shell", target_os = "linux"))]
fn place_with_layer_shell(window: &ApplicationWindow, position: Position, width: i32, height: i32) {
    use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

    if !gtk4_layer_shell::is_supported() {
        return;
    }
    window.init_layer_shell();
    window.set_namespace(Some("desk-monitoring"));
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_exclusive_zone(-1);

    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        window.set_anchor(edge, false);
        window.set_margin(edge, 0);
    }

    let (mw, mh) = monitor_size().unwrap_or((1920, 1080));
    match position {
        Position::Left => {
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Left, 12);
            window.set_margin(Edge::Top, ((mh - height) / 2).max(12));
        }
        Position::Right => {
            window.set_anchor(Edge::Right, true);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Right, 12);
            window.set_margin(Edge::Top, ((mh - height) / 2).max(12));
        }
        Position::Top => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Top, 12);
            window.set_margin(Edge::Left, ((mw - width) / 2).max(12));
        }
        Position::Bottom => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Bottom, 12);
            window.set_margin(Edge::Left, ((mw - width) / 2).max(12));
        }
        Position::Center => {
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Left, ((mw - width) / 2).max(12));
            window.set_margin(Edge::Top, ((mh - height) / 2).max(12));
        }
    }
}

fn monitor_size() -> Option<(i32, i32)> {
    let display = Display::default()?;
    let monitor = display.monitors().item(0)?.downcast::<gdk::Monitor>().ok()?;
    let geo = monitor.geometry();
    Some((geo.width(), geo.height()))
}

pub fn attach_drag(window: &ApplicationWindow, surface_host: &impl IsA<gtk::Widget>) {
    let click = gtk::GestureClick::new();
    click.set_button(gdk::BUTTON_PRIMARY);
    let window_weak = window.downgrade();
    click.connect_pressed(move |gesture, n_press, x, y| {
        if n_press != 1 {
            return;
        }
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        let Some(native) = window.native() else {
            return;
        };
        let Some(surface) = native.surface() else {
            return;
        };
        let Ok(toplevel) = surface.downcast::<gdk::Toplevel>() else {
            return;
        };
        let Some(device) = gesture.device() else {
            return;
        };
        toplevel.begin_move(&device, gdk::BUTTON_PRIMARY as i32, x, y, gdk::CURRENT_TIME);
    });
    surface_host.add_controller(click);
}
