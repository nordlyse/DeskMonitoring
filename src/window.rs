use std::sync::{Arc, Mutex};
use std::time::Duration;

use gtk::gdk;
use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea, Orientation};

use crate::config::Config;
use crate::hud;
use crate::metrics::{self, Collector};
use crate::overlay;
use crate::setup;
use crate::snapshot::Snapshot;

pub fn show_monitor(app: &Application, config: Config) {
    for window in app.windows() {
        if window.has_css_class("desk-monitor") {
            window.present();
            return;
        }
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desk Monitor")
        .build();
    overlay::apply_window_chrome(&window);
    overlay::place_overlay(&window, &config);

    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let live = Arc::new(Mutex::new(config.clone()));
    let mut collector = Collector::add();
    collector.refresh_local(&snapshot);
    metrics::spawn_remote_loop(live.clone(), snapshot.clone());

    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.add_css_class("desk-monitor-canvas");

    let draw_live = live.clone();
    let draw_snapshot = snapshot.clone();
    area.set_draw_func(move |_, cr, width, height| {
        let snap = metrics::copy_snapshot(&draw_snapshot);
        let config = draw_live.lock().map(|guard| guard.clone()).unwrap_or_default();
        hud::paint(cr, width, height, &snap, &config);
    });

    let host = gtk::Box::new(Orientation::Vertical, 0);
    host.add_css_class("desk-monitor-host");
    host.append(&area);
    window.set_child(Some(&host));
    attach_pointer(&window, &area, live.clone());

    let metric_area = area.clone();
    let metric_snapshot = snapshot.clone();
    let mut collector = collector;
    glib::timeout_add_local(Duration::from_secs(1), move || {
        collector.refresh_local(&metric_snapshot);
        metric_area.queue_draw();
        ControlFlow::Continue
    });

    window.present();
}

fn attach_pointer(window: &ApplicationWindow, area: &DrawingArea, live: Arc<Mutex<Config>>) {
    let drag = gtk::GestureClick::new();
    drag.set_button(gdk::BUTTON_PRIMARY);
    let window_weak = window.downgrade();
    let area_click = area.clone();
    let live_click = live.clone();
    drag.connect_pressed(move |gesture, n_press, x, y| {
        if n_press != 1 {
            return;
        }
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        let config = live_click.lock().map(|guard| guard.clone()).unwrap_or_default();
        if hud::hit_settings(x, y, area_click.width(), config.position) {
            if let Some(app) = window.application() {
                setup::show_settings(&app, Some(live_click.clone()));
            }
            return;
        }
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
    area.add_controller(drag);

    let menu = gtk::GestureClick::new();
    menu.set_button(gdk::BUTTON_SECONDARY);
    let window_weak = window.downgrade();
    menu.connect_pressed(move |_, n_press, _, _| {
        if n_press != 1 {
            return;
        }
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if let Some(app) = window.application() {
            setup::show_settings(&app, Some(live.clone()));
        }
    });
    area.add_controller(menu);
}
