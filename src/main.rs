#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, time::Duration};
use sysinfo::System;
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

const GB: f64 = 1024.0 * 1024.0 * 1024.0;

enum UserEvent {
    Tick(String),
    Menu(MenuEvent),
}

fn tooltip(used: u64, total: u64) -> String {
    let pct = used as f64 / total as f64 * 100.0;
    format!("{:.1} / {:.1} GB  {:.0}%", used as f64 / GB, total as f64 / GB, pct)
}

// shortcut: solid 16x16 square in the accent color, upgrade to a drawn icon when the look matters
fn icon() -> Icon {
    let rgba = [0x00, 0x78, 0xD4, 0xFF].repeat(16 * 16);
    Icon::from_rgba(rgba, 16, 16).expect("icon buffer is 16x16 RGBA")
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));

    let proxy = event_loop.create_proxy();
    thread::spawn(move || {
        let mut sys = System::new().expect("sysinfo: unsupported platform");
        loop {
            sys.refresh_memory();
            let text = tooltip(sys.used_memory(), sys.total_memory());
            if proxy.send_event(UserEvent::Tick(text)).is_err() {
                break; // event loop gone, app is exiting
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    let menu = Menu::new();
    let quit = MenuItem::new("Quit", true, None);
    menu.append(&quit).expect("append Quit");

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("RAM")
        .with_icon(icon())
        .build()
        .expect("create tray icon");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(UserEvent::Tick(text)) => {
                let _ = tray.set_tooltip(Some(text));
            }
            Event::UserEvent(UserEvent::Menu(e)) if e.id == quit.id() => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_shows_used_total_and_percent() {
        assert_eq!(tooltip(12_241_000_000, 34_250_000_000), "11.4 / 31.9 GB  36%");
    }
}
