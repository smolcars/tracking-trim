#![cfg_attr(windows, windows_subsystem = "windows")]

mod clean;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

static PAUSED: AtomicBool = AtomicBool::new(false);

fn icon() -> Icon {
    const SIZE: i32 = 32;
    let rgba = (0..SIZE * SIZE)
        .flat_map(|i| {
            // Doubled coordinates put the centre of the even-sized grid on an integer.
            let (x, y) = (2 * (i % SIZE) + 1 - SIZE, 2 * (i / SIZE) + 1 - SIZE);
            let alpha = if x * x + y * y <= SIZE * SIZE {
                0xff
            } else {
                0
            };
            [0x2e, 0xa0, 0x43, alpha]
        })
        .collect();
    Icon::from_rgba(rgba, SIZE as u32, SIZE as u32).expect("icon is 32x32 RGBA")
}

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
        last = match clean::clean(&text) {
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

    let pause = CheckMenuItem::new("Pause", true, false, None);
    let quit = MenuItem::new("Quit", true, None);
    let (pause_id, quit_id) = (pause.id().clone(), quit.id().clone());
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
    }));

    let tray = TrayIconBuilder::new()
        .with_icon(icon())
        .with_tooltip("tracking-trim")
        .with_menu(Box::new(Menu::with_items(&[&pause, &quit]).expect("menu")))
        .build();
    // Linux keeps cleaning without a session bus; elsewhere no tray means no way to quit.
    if let Err(e) = &tray {
        eprintln!("tracking-trim: no tray: {e}");
        #[cfg(not(target_os = "linux"))]
        std::process::exit(1);
    }

    // ksni runs its own thread on Linux; macOS and Windows need a run loop on this one.
    #[cfg(target_os = "linux")]
    poll();
    #[cfg(not(target_os = "linux"))]
    std::thread::spawn(poll);
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

#[cfg(test)]
mod tests {
    #[test]
    fn icon_is_valid_rgba() {
        super::icon();
    }
}
