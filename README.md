<img src="assets/icon.svg" width="64" height="64" alt="">

# tracking-trim

A tray app that watches the clipboard and, when it holds a single http(s) URL, rewrites it with the tracking parameters removed. Anything else on the clipboard is left alone. There is no network access, and the only settings are the toggles in the tray menu.

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

Start the binary and it lives in the tray.

On macOS 15.4+ you may see a one-time "paste from other apps" prompt. Choose Allow Always.

## Menu

- **Pause:** stops cleaning until you uncheck it.
- **Sites:** one checkbox per site (Instagram, X, YouTube, TikTok, Threads, Reddit), all on by default. Links on an unchecked site are left completely untouched.
- **X links → fixupx.com:** off by default. When on, X and Twitter post links also move to `fixupx.com`.
- **Launch at login:** off by default. When on, the app starts at your next login. See [Launch at login](#launch-at-login).
- **Quit**

The Sites and fixupx toggles are saved in a plain `key=value` file:

- **macOS:** `~/Library/Application Support/tracking-trim/settings`
- **Linux:** `$XDG_CONFIG_HOME/tracking-trim/settings`, or `~/.config/tracking-trim/settings` if that is unset
- **Windows:** `%APPDATA%\tracking-trim\settings`

## Linux notes

- **X11:** works.
- **Wayland on KDE, Sway and Hyprland:** works natively through data-control.
- **Wayland on GNOME:** works through the XWayland clipboard bridge, so XWayland must be enabled.
- **Tray icon on GNOME:** needs the AppIndicator extension. Cleaning runs regardless.

## Launch at login

The menu toggle registers the binary with the OS. macOS may show an "App Background Activity" notification when you turn it on; no action is needed. That registration is the only record of it, so to remove it by hand:

- **macOS:** turn it off under System Settings > General > Login Items & Extensions (Login Items before macOS 15) > Allow in the Background, or delete `~/Library/LaunchAgents/com.smolcars.tracking-trim.plist`.
- **Windows:** delete the `tracking-trim` value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **Linux:** delete `$XDG_CONFIG_HOME/autostart/tracking-trim.desktop`, or `~/.config/autostart/tracking-trim.desktop` if that is unset.

The registration stores the binary's full path, so moving the binary breaks it. After moving it, toggle Launch at login off and on again.

## Releases

Release notes come from `CHANGELOG.md`: add a `## vX.Y.Z` section before tagging, or the release job fails.

Every `v*` tag publishes release archives to GitHub Releases. The binaries are unsigned and not notarized; to avoid that, build from source as described under [Build](#build).

| Platform | Architecture | Asset |
|---|---|---|
| macOS | arm64 (Apple silicon) | `tracking-trim-macos-arm64.tar.gz` |
| Windows | x86_64 | `tracking-trim-windows-x86_64.zip` |
| Linux | x86_64 | `tracking-trim-linux-x86_64.tar.gz` |
| Linux | arm64 | `tracking-trim-linux-arm64.tar.gz` |

On first run:

- **macOS:** Gatekeeper blocks unsigned apps. After the first blocked launch, go to System Settings > Privacy & Security and choose Open Anyway, or run `xattr -d com.apple.quarantine tracking-trim`.
- **Windows:** SmartScreen warns about the binary. Choose More info > Run anyway.

## CI

Every push to `main` builds and tests on Blacksmith runners for Linux (x86_64 and arm64), Windows and macOS. Each run uploads the release binary as an artifact per OS and architecture (`tracking-trim-Linux-X64`, `tracking-trim-Linux-ARM64`, `tracking-trim-Windows-X64`, `tracking-trim-macOS-ARM64`), kept for 14 days. A `v*` tag runs the same jobs and also publishes the [release archives](#releases).
