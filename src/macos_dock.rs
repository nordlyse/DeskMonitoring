#![allow(unexpected_cfgs)]

use std::ffi::CString;
use std::sync::OnceLock;

use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use gtk::glib;
use gtk::prelude::*;

use crate::config;
use crate::setup;
use crate::window;

struct NsPtr(*mut Object);
unsafe impl Send for NsPtr {}
unsafe impl Sync for NsPtr {}

const TAG_SETTINGS: isize = 0;
const TAG_MATRIX: isize = 1;
const TAG_TURQUOISE: isize = 2;
const TAG_BLUE: isize = 3;
const TAG_PINK: isize = 4;
const TAG_YELLOW: isize = 5;

pub fn install_dock_menu() {
    static MENU: OnceLock<NsPtr> = OnceLock::new();
    if MENU.get().is_some() {
        return;
    }
    unsafe {
        let nsapp: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        if nsapp.is_null() {
            return;
        }
        let target = dock_target();
        let options: *mut Object = msg_send![class!(NSMenu), new];
        add_item(options, target, "Settings", TAG_SETTINGS);
        let sep: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![options, addItem: sep];
        add_item(options, target, "Matrix green", TAG_MATRIX);
        add_item(options, target, "Turquoise", TAG_TURQUOISE);
        add_item(options, target, "Blue", TAG_BLUE);
        add_item(options, target, "Pink", TAG_PINK);
        add_item(options, target, "Yellow", TAG_YELLOW);

        let dock: *mut Object = msg_send![class!(NSMenu), new];
        let options_item: *mut Object = msg_send![class!(NSMenuItem), alloc];
        let empty = nsstring("");
        let options_item: *mut Object = msg_send![
            options_item,
            initWithTitle: nsstring("Options")
            action: sel!(pick:)
            keyEquivalent: empty
        ];
        let _: () = msg_send![options_item, setSubmenu: options];
        let _: () = msg_send![dock, addItem: options_item];
        let _: () = msg_send![nsapp, setDockMenu: dock];
        let _ = MENU.set(NsPtr(dock));
    }
}

fn add_item(menu: *mut Object, target: *mut Object, title: &str, tag: isize) {
    unsafe {
        let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
        let item: *mut Object = msg_send![
            item,
            initWithTitle: nsstring(title)
            action: sel!(pick:)
            keyEquivalent: nsstring("")
        ];
        let _: () = msg_send![item, setTarget: target];
        let _: () = msg_send![item, setTag: tag];
        let _: () = msg_send![menu, addItem: item];
    }
}

fn nsstring(value: &str) -> *mut Object {
    let cstr = CString::new(value).unwrap_or_else(|_| CString::new("").unwrap());
    unsafe { msg_send![class!(NSString), stringWithUTF8String: cstr.as_ptr()] }
}

fn dock_target() -> *mut Object {
    static TARGET: OnceLock<NsPtr> = OnceLock::new();
    TARGET
        .get_or_init(|| unsafe {
            let cls = dock_class();
            let obj: *mut Object = msg_send![cls, new];
            NsPtr(obj)
        })
        .0
}

fn dock_class() -> &'static Class {
    if let Some(cls) = Class::get("DeskMonitorDockTarget") {
        return cls;
    }
    let mut decl = ClassDecl::new("DeskMonitorDockTarget", class!(NSObject))
        .expect("DeskMonitorDockTarget");
    unsafe {
        decl.add_method(
            sel!(pick:),
            on_pick as extern "C" fn(&Object, Sel, *mut Object),
        );
    }
    decl.register()
}

extern "C" fn on_pick(_this: &Object, _cmd: Sel, sender: *mut Object) {
    let tag: isize = unsafe { msg_send![sender, tag] };
    glib::idle_add_once(move || apply_tag(tag));
}

fn apply_tag(tag: isize) {
    let Some(gio_app) = gtk::gio::Application::default() else {
        return;
    };
    let Ok(app) = gio_app.downcast::<gtk::Application>() else {
        return;
    };
    if tag == TAG_SETTINGS {
        setup::show_settings(&app, Some(config::live_slot()));
        return;
    }
    let kind = match tag {
        TAG_TURQUOISE => config::PaletteKind::Turquoise,
        TAG_BLUE => config::PaletteKind::Blue,
        TAG_PINK => config::PaletteKind::Pink,
        TAG_YELLOW => config::PaletteKind::Yellow,
        _ => config::PaletteKind::Matrix,
    };
    let mut cfg = config::current_config();
    cfg.palette = kind;
    let _ = config::write_config(&cfg);
    window::refresh_overlay(&app);
}
