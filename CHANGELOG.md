# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Volume feedback sound** — a short system sound plays after releasing the output slider so the user can judge whether the new level is adequate. Uses `paplay` with the freedesktop sound theme (`audio-volume-change.oga`); silently no-ops if `paplay` or the sound file is not available. The sound is applied after `wpctl set-volume` completes so it plays at the new level.

## [0.1.0] - 2026-03-10

### Added
- **Output and input volume sliders** (0–150%) backed by `wpctl set-volume` on release.
- **Mute All toggle** — mutes/unmutes both sink and source simultaneously via `wpctl set-mute toggle`; button turns accent colour when active.
- **PipeWire abstraction** (`src/audio/`) — `load`, `set_sink_volume`, `set_source_volume`, and `toggle_mute_all` isolated from the UI so future components can share them without reimplementing `wpctl` calls.
- **Theme system** matching trebuchet's conventions: `Theme` struct with named colour fields, `Default` built from hex literals, `Theme::load()` reading `~/.config/veu/theme.conf` with a three-layer resolution (compiled defaults → user `theme.conf` → named theme via `current-theme` pointer file).
- **Bundled themes**: `default`, `catppuccin-mocha`, `dracula`, `gruvbox-dark`, `nord`, `tokyo-night`.
- **`scripts/install.sh`** — system-wide (`/usr/local/bin`) or user (`~/.local/bin`) install, with `--uninstall`, `--yes`, and `--no` flags; installs binary, default theme config, bundled themes, and a `.desktop` entry.
- Closes on **Escape** key or **click outside** the popup.
- Wayland layer-shell overlay centred by the compositor, 380 × 180 px, keyboard-exclusive.
