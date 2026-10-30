# tracking-trim engineering plan

Background tray app that rewrites the clipboard whenever it holds a single
http(s) URL with tracking junk. One Rust binary, macOS + Windows + Linux.

## Stack (all current as of 2026-10-30, Rust 1.97 local)

| Crate | Ver | Role | Why |
|---|---|---|---|
| arboard | 3.7 | clipboard read/write | one API on all 3 OSes, Wayland data-control + X11/XWayland fallback built in |
| tray-icon | 0.26 | tray icon + menu | Windows, macOS; `ksni` feature on Linux so no GTK/libappindicator build deps |

URL splitting is hand-rolled on std (only whole `key=value` segments are ever
deleted, nothing is decoded or re-encoded). The run loop tray-icon needs on
macOS/Windows comes from crates it already pulls in (`objc2-app-kit`
NSApplication, `windows-sys` GetMessage loop); Linux's ksni backend runs its own
thread. No settings file, no UI window, no network.

## Files

```
Cargo.toml
src/clean.rs   pure fn clean(&str) -> Option<String> + rules table + tests   (~100 lines)
src/main.rs    tray + poll loop                                              (~80 lines)
README.md      build/run per OS, Linux notes, autostart notes
```

## Cleaning rules (src/clean.rs)

Input must be one trimmed http/https URL, no inner whitespace. Otherwise `None`.

1. **Path-ID sites: drop every query param except an explicit keep list.**
   The path already names the content, so rotating param names can never
   break this. This is what beats Instagram's `mdxt`/`psln`/... rotation.

   | host (suffix match, `www.`/`m.`/`mobile.` ok) | path contains | keep |
   |---|---|---|
   | instagram.com | `/p/` `/reel/` `/reels/` `/tv/` | none |
   | x.com, twitter.com | `/status/` | none |
   | youtu.be | any | `t` |
   | youtube.com | `/shorts/` | none |
   | youtube.com | `/watch` | `v` `t` `list` `index` |
   | tiktok.com | `/video/` | none |
   | threads.net, threads.com | `/post/` | none |
   | reddit.com | `/comments/` | none |

2. **Everything else: drop a global denylist**, keep the rest.
   `utm_*` prefix, `fbclid`, `gclid`, `gclsrc`, `dclid`, `gbraid`, `wbraid`,
   `msclkid`, `ttclid`, `twclid`, `igsh`, `igshid`, `mc_cid`, `mc_eid`,
   `ref_src`, `ref_url`, `_ga`, `_gl`, `si`.

3. Return `None` when nothing changed, so the clipboard is never rewritten
   needlessly. Fragment (`#...`) is left alone.

Tests (same file, `cargo test`): the two sample links from the brief, a
youtube watch link keeping `v`+`t`, a youtu.be link keeping `t`, a clean URL
(`None`), prose containing a URL (`None`), a non-URL (`None`), a mailto (`None`).

## Runtime loop (src/main.rs)

- `#![windows_subsystem = "windows"]` so no console on Windows.
- macOS: NSApplication run loop on the main thread with activation policy
  Accessory, so there is no Dock icon even as a bare binary. Windows: Win32
  message loop. Linux: none needed.
- Tray icon: 32x32 RGBA drawn in code (filled circle), no image file, no
  `image` crate. Menu: `Pause` / `Resume` toggle, `Quit`.
- Background thread: every 250 ms `arboard.get_text()`; skip if paused or
  equal to last seen; if `clean()` returns `Some(c)`, `set_text(c)` and
  remember `c` as last seen so we never loop on our own write.
- `shortcut:` polling reads full clipboard text each tick; upgrade to
  NSPasteboard changeCount / WM_CLIPBOARDUPDATE only if CPU shows up.

## Build / run

```
cargo test
cargo run --release          # macOS, Windows
cargo run --release          # Linux (X11, KDE/Sway/Hyprland Wayland natively,
                             #   GNOME Wayland via XWayland bridge)
```

Linux tray: KDE shows it as-is, GNOME needs the AppIndicator extension; the
cleaning runs regardless.

macOS 15.4+: first read may show a "paste from other apps" prompt, pick
Allow Always.

## Verify

1. `cargo test` green.
2. `cargo run --release`, tray icon appears, no Dock icon.
3. Copy the Instagram and X sample links, `pbpaste` prints the bare URL.
4. Copy `https://www.youtube.com/watch?v=abc&t=42&si=xyz`, `pbpaste` keeps `v` and `t`.
5. Copy a paragraph with a URL in it, clipboard unchanged.
6. Pause from the tray, copy a tracked link, clipboard unchanged. Resume, works.

## CI

Blacksmith runners for all three OSes, `Swatinem/rust-cache` on every job,
release profile only, binaries uploaded as artifacts. Push to `main` only,
after local verification.

## Deliberately not in v1

- Launch at login (README gives the manual step per OS).
- `.app` bundle, code signing, notarization.
- Custom rule file, settings window, clipboard history, "restore original".
- Short-link expansion (needs network).
