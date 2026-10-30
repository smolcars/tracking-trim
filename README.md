<img src="assets/icon.svg" width="64" height="64" alt="">

# tracking-trim

A tray app that watches the clipboard and, when it holds a single http(s) URL, rewrites it with the tracking parameters removed. Anything else on the clipboard is left alone. There is no network access and there are no settings.

Two kinds of rules apply:

- **Path-based:** Instagram `/p` and `/reel`, X `/status`, YouTube, TikTok, Threads and Reddit links keep only the parameters that select the content, so renamed tracking parameters still get removed.
- **Global denylist:** every other URL loses `utm_*`, `fbclid`, `gclid` and similar parameters and keeps the rest.

## Build

```sh
nix develop
cargo build --release
```

Or use plain cargo with Rust 1.97.1. The binary is written to `target/release/`.

## Run

Start the binary and it lives in the tray. The menu has Pause/Resume and Quit.

On macOS 15.4+ you may see a one-time "paste from other apps" prompt. Choose Allow Always.

## Linux notes

- **X11:** works.
- **Wayland on KDE, Sway and Hyprland:** works natively through data-control.
- **Wayland on GNOME:** works through the XWayland clipboard bridge, so XWayland must be enabled.
- **Tray icon on GNOME:** needs the AppIndicator extension. Cleaning runs regardless.

## Launch at login

- **macOS:** add the binary under System Settings > General > Login Items.
- **Windows:** press Win+R, open `shell:startup`, and put a shortcut to `tracking-trim.exe` in that folder.
- **Linux:** create `~/.config/autostart/tracking-trim.desktop`, using the full path to the binary:

```ini
[Desktop Entry]
Type=Application
Name=tracking-trim
Exec=/path/to/tracking-trim
```

## CI

Every push to `main` builds and tests on Blacksmith runners for Linux, Windows and macOS. Each run uploads the release binary as an artifact per OS (`tracking-trim-Linux`, `tracking-trim-Windows`, `tracking-trim-macOS`), kept for 14 days.
