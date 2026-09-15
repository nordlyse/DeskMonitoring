use std::sync::{Arc, Mutex};
use std::time::Duration;

use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea, Orientation};

use crate::config::Config;
use crate::hud;
use crate::metrics::{self, Collector};
use crate::overlay;
use crate::snapshot::Snapshot;
use crate::theme::Palette;

pub fn show_monitor(app: &Application, config: Config) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desk Monitor")
        .build();
    overlay::apply_window_chrome(&window);
    overlay::place_overlay(&window, &config);

    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let mut collector = Collector::add();
    collector.refresh_local(&snapshot);
    metrics::spawn_remote_loop(config.clone(), snapshot.clone());

    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.add_css_class("desk-monitor-canvas");

    let draw_config = config.clone();
    let draw_snapshot = snapshot.clone();
    area.set_draw_func(move |_, cr, width, height| {
        let snap = metrics::copy_snapshot(&draw_snapshot);
        let palette = Palette::from_kind(draw_config.palette);
        hud::paint(cr, width, height, &snap, palette, draw_config.position);
    });

    let host = gtk::Box::new(Orientation::Vertical, 0);
    host.add_css_class("desk-monitor-host");
    host.append(&area);
    window.set_child(Some(&host));
    overlay::attach_drag(&window, &area);

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
