# Changelog

Release notes are taken from this file: the section whose heading matches the
tag becomes the GitHub release body.

## v0.1.0

First release.

- Watches the clipboard and, when it holds a single link, strips the tracking
  parameters. Instagram, X, YouTube, TikTok, Threads and Reddit links keep only
  the parameters that select the content, so rotating tracker names are caught
  too. Every other site loses `utm_*`, `fbclid`, `gclid` and similar.
- Tray menu: Pause, one on/off switch per site, "X links → fixupx.com",
  "Launch at login", Quit. Site and fixupx choices persist in a small settings
  file.
- Unsigned binaries for macOS (arm64), Windows (x86_64) and Linux (x86_64 and
  arm64). See the README for the first-run steps on macOS and Windows.
