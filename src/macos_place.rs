#![allow(unexpected_cfgs)]

use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

use crate::config::Position;

#[repr(C)]
#[derive(Clone, Copy)]
struct NsPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NsSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NsRect {
    origin: NsPoint,
    size: NsSize,
}

extern "C" {
    fn gdk_macos_surface_get_native_window(
        surface: *mut gtk::gdk::ffi::GdkSurface,
    ) -> *mut Object;
}

pub fn apply_frame(window: &ApplicationWindow, position: Position, width: i32, height: i32) {
    let Some(surface) = window.surface() else {
        return;
    };
    unsafe {
        let native = gdk_macos_surface_get_native_window(surface.to_glib_none().0);
        if native.is_null() {
            return;
        }
        let screen: *mut Object = msg_send![native, screen];
        if screen.is_null() {
            return;
        }
        let vis: NsRect = msg_send![screen, visibleFrame];
        let margin = 12.0;
        let w = width as f64;
        let h = height as f64;
        let x = match position {
            Position::Left => vis.origin.x + margin,
            Position::Right => vis.origin.x + vis.size.width - w - margin,
            Position::Top | Position::Bottom | Position::Center => {
                vis.origin.x + ((vis.size.width - w) * 0.5).max(margin)
            }
        };
        let y = match position {
            Position::Top => vis.origin.y + vis.size.height - h - margin,
            Position::Bottom => vis.origin.y + margin,
            Position::Left | Position::Right | Position::Center => {
                vis.origin.y + ((vis.size.height - h) * 0.5).max(0.0)
            }
        };
        let rect = NsRect {
            origin: NsPoint { x, y },
            size: NsSize { width: w, height: h },
        };
        let _: () = msg_send![native, setFrame: rect display: true];
    }
}
