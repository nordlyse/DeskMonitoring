use gtk::gdk::{self, Display, prelude::SurfaceExt};
use gtk::glib;
use gtk::prelude::*;
use gtk::ApplicationWindow;

use crate::config::{Config, Position};

const PANEL_WIDTH: i32 = 380;
const PANEL_MARGIN: i32 = 12;

pub fn panel_size(screen_w: i32, screen_h: i32) -> (i32, i32) {
    let width = PANEL_WIDTH.min((screen_w as f64 * 0.32) as i32).max(300);
    let height = (screen_h - PANEL_MARGIN * 2)
        .min((screen_h as f64 * 0.92) as i32)
        .max(400);
    let _ = screen_w;
    (width, height)
}

pub fn apply_window_chrome(window: &ApplicationWindow) {
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_title(Some("Desk Monitor"));
    window.add_css_class("desk-monitor");
    window.add_css_class("undecorated");
    window.connect_realize(|window| {
        if let Some(surface) = window.surface() {
            surface.set_opaque_region(None);
        }
    });
    window.connect_map(|window| {
        let position = crate::config::current_config().position;
        snap_overlay(window, position);
    });
}

pub fn place_overlay(window: &ApplicationWindow, config: &Config) {
    let (mw, mh) = monitor_size().unwrap_or((1440, 900));
    let (width, height) = panel_size(mw, mh);
    window.set_resizable(true);
    window.set_default_size(width, height);
    window.set_size_request(width, height);
    window.set_resizable(false);

    #[cfg(all(feature = "layer-shell", target_os = "linux"))]
    {
        place_with_layer_shell(window, config.position, width, height);
    }

    let position = config.position;
    let window = window.clone();
    glib::idle_add_local_once(move || {
        snap_overlay(&window, position);
    });
}

fn snap_overlay(window: &ApplicationWindow, position: Position) {
    let (mw, mh) = monitor_size().unwrap_or((1440, 900));
    let (width, height) = panel_size(mw, mh);
    window.set_resizable(true);
    window.set_default_size(width, height);
    window.set_size_request(width, height);
    window.set_resizable(false);

    #[cfg(target_os = "macos")]
    crate::macos_place::apply_frame(window, position, width, height);

    #[cfg(not(target_os = "macos"))]
    {
        let _ = position;
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
            window.set_margin(Edge::Left, PANEL_MARGIN);
            window.set_margin(Edge::Top, ((mh - height) / 2).max(PANEL_MARGIN));
        }
        Position::Right => {
            window.set_anchor(Edge::Right, true);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Right, PANEL_MARGIN);
            window.set_margin(Edge::Top, ((mh - height) / 2).max(PANEL_MARGIN));
        }
        Position::Top => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Top, PANEL_MARGIN);
            window.set_margin(Edge::Left, ((mw - width) / 2).max(PANEL_MARGIN));
        }
        Position::Bottom => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Bottom, PANEL_MARGIN);
            window.set_margin(Edge::Left, ((mw - width) / 2).max(PANEL_MARGIN));
        }
        Position::Center => {
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Left, ((mw - width) / 2).max(PANEL_MARGIN));
            window.set_margin(Edge::Top, ((mh - height) / 2).max(PANEL_MARGIN));
        }
    }
}

fn monitor_size() -> Option<(i32, i32)> {
    let display = Display::default()?;
    let monitor = display.monitors().item(0)?.downcast::<gdk::Monitor>().ok()?;
    let geo = monitor.geometry();
    Some((geo.width(), geo.height()))
}
