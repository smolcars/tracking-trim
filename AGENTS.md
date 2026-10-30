# AGENTS.md

tracking-trim is a Rust system-tray app for macOS, Windows and Linux. It watches
the clipboard and, when the clipboard holds a single http(s) URL, rewrites it
with tracking parameters removed. See PLAN.md for the design and rules table.

## Rules

- Minimal dependencies. Use std first. Every crate must be justified; no crate
  for a few lines of code. Current direct deps: `arboard` (clipboard),
  `tray-icon` (tray). Platform run loops use crates tray-icon already pulls in.
- All tooling comes from `flake.nix` (`nix develop`). Never tell anyone to
  brew/apt install something.
- CI runs on Blacksmith runners and every Rust job caches with
  `Swatinem/rust-cache`. Never add an uncached Rust job.
- No PRs. Push straight to `main`, and only after the work is complete and
  proven locally (`cargo fmt --check`, `cargo clippy -- -D warnings`,
  `cargo test`, and a manual run on the host OS).
- If you create a git worktree, remove it when you are done.
- Cleaning rules live in `src/clean.rs`. Prefer path-based "drop every query
  param" rules for sites whose path already identifies the content; use the
  global denylist only for everything else. Add a unit test for every new rule.
- No network calls, no clipboard history. Leave the clipboard untouched when in
  doubt.
- Settings are one plain key=value file in the OS config dir, changed only
  through the tray menu; no settings window.

## Layout

```
src/clean.rs     pure clean(&str, &Settings) -> Option<String>, rules, unit tests
src/settings.rs  Site enum, Settings, load/save of the key=value file
src/main.rs      tray icon, clipboard poll loop, per-OS run loop
flake.nix      dev shell (toolchain pinned by rust-toolchain.toml)
.github/workflows/ci.yml   Blacksmith matrix, cached, uploads binaries
```

## Verify

```
cargo test
cargo build --release
./target/release/tracking-trim &   # then copy a tracked link and paste it
```

macOS can be tested live here. Linux and Windows are checked with
`cargo check --target x86_64-unknown-linux-gnu` / `x86_64-pc-windows-gnu`
and by CI.
