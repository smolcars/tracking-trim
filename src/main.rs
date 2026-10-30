#![cfg_attr(windows, windows_subsystem = "windows")]

mod autostart;
mod clean;
mod icon;
mod settings;

use settings::{Settings, Site};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tray_icon::TrayIconBuilder;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, Submenu};

static PAUSED: AtomicBool = AtomicBool::new(false);
static SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| Mutex::new(Settings::load()));

// Menu items are !Send, and the handler runs on the main thread on macOS and Windows.
#[cfg(not(target_os = "linux"))]
thread_local!(static LOGIN: std::cell::OnceCell<CheckMenuItem> = const { std::cell::OnceCell::new() });

fn poll() {
    let mut clipboard = arboard::Clipboard::new().unwrap_or_else(|e| {
        eprintln!("tracking-trim: no clipboard: {e}");
        std::process::exit(1)
    });
    let mut last = String::new();
    // shortcut: reads full clipboard text every tick; switch to change-count/WM_CLIPBOARDUPDATE if CPU shows up
    loop {
        std::thread::sleep(Duration::from_millis(250));
        let Ok(text) = clipboard.get_text() else {
            continue;
        };
        if text == last {
            continue;
        }
        // Paused still tracks the clipboard, so resuming never rewrites an older copy.
        if PAUSED.load(Ordering::Relaxed) {
            last = text;
            continue;
        }
        let settings = SETTINGS.lock().unwrap().clone();
        last = match clean::clean(&text, &settings) {
            Some(c) => {
                // A failed write leaves the clipboard != last, so the next tick retries.
                let _ = clipboard.set_text(c.clone());
                c
            }
            None => text,
        };
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    let app = {
        let mtm = objc2::MainThreadMarker::new().expect("main runs on the main thread");
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        // Accessory: no Dock icon, even as a bare binary.
        app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
        app
    };

    let saved = SETTINGS.lock().unwrap().clone();
    let pause = CheckMenuItem::new("Pause", true, false, None);
    let sites = Submenu::new("Sites", true);
    let site_items: Vec<_> = Site::NAMES
        .iter()
        .zip(saved.sites)
        .map(|(name, on)| CheckMenuItem::new(name, true, on, None))
        .collect();
    for item in &site_items {
        sites.append(item).expect("menu");
    }
    let fixupx = CheckMenuItem::new("X links → fixupx.com", true, saved.fixupx, None);
    let login = CheckMenuItem::new("Launch at login", true, autostart::is_enabled(), None);
    let quit = MenuItem::new("Quit", true, None);
    let (pause_id, quit_id) = (pause.id().clone(), quit.id().clone());
    let site_ids: Vec<_> = site_items.iter().map(|item| item.id().clone()).collect();
    let (fixupx_id, login_id) = (fixupx.id().clone(), login.id().clone());
    #[cfg(target_os = "linux")]
    let (login_tx, login_rx) = std::sync::mpsc::channel();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == quit_id {
            // Ending the message loop drops the tray, which removes the Windows icon.
            #[cfg(windows)]
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage(0)
            };
            #[cfg(not(windows))]
            std::process::exit(0);
        }
        if event.id == pause_id {
            // The item flips its own checkmark on every OS; this mirrors it.
            PAUSED.fetch_xor(true, Ordering::Relaxed);
        }
        if event.id == login_id {
            let result = if autostart::is_enabled() {
                autostart::disable()
            } else {
                autostart::enable()
            };
            if let Err(e) = result {
                eprintln!("tracking-trim: launch at login: {e}");
            }
            // The item flipped itself, so always re-read: the OS registration is the truth.
            #[cfg(target_os = "linux")]
            let _ = login_tx.send(());
            #[cfg(not(target_os = "linux"))]
            LOGIN.with(|login| {
                let login = login.get().expect("set before the run loop");
                login.set_checked(autostart::is_enabled());
            });
        }
        let site = site_ids.iter().position(|id| *id == event.id);
        if site.is_some() || event.id == fixupx_id {
            let mut settings = SETTINGS.lock().unwrap();
            match site {
                Some(i) => settings.sites[i] ^= true,
                None => settings.fixupx ^= true,
            }
            let _ = settings.save();
        }
    }));

    let builder = TrayIconBuilder::new()
        .with_tooltip("tracking-trim")
        .with_menu(Box::new(
            Menu::with_items(&[&pause, &sites, &fixupx, &login, &quit]).expect("menu"),
        ));
    // Template: macOS recolors the alpha mask for light and dark menu bars.
    #[cfg(target_os = "macos")]
    let builder = builder.with_icon_templated(icon::tray_icon());
    #[cfg(not(target_os = "macos"))]
    let builder = builder.with_icon(icon::tray_icon());
    let tray = builder.build();
    // Linux keeps cleaning without a session bus; elsewhere no tray means no way to quit.
    if let Err(e) = &tray {
        eprintln!("tracking-trim: no tray: {e}");
        #[cfg(not(target_os = "linux"))]
        std::process::exit(1);
    }

    std::thread::spawn(poll);
    // ksni runs the menu on its own thread on Linux, so this one only owns the login item;
    // macOS and Windows need a run loop here.
    #[cfg(target_os = "linux")]
    for () in login_rx {
        login.set_checked(autostart::is_enabled());
    }
    #[cfg(not(target_os = "linux"))]
    let _ = LOGIN.with(|cell| cell.set(login));
    #[cfg(target_os = "macos")]
    app.run();
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage,
        };
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
