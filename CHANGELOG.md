# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Volume feedback sound** — a short system sound plays after releasing the output slider so the user can judge whether the new level is adequate. Uses `paplay` with the freedesktop sound theme (`audio-volume-change.oga`); silently no-ops if `paplay` or the sound file is not available. The sound is applied after `wpctl set-volume` completes so it plays at the new level.
- **Configurable placement** — `placement` and `margin` are now first-class theme fields. The popup can be anchored to any corner or edge (`top-right`, `top-left`, `top-center`, `bottom-right`, `bottom-left`, `bottom-center`, `center`). Both keys live in `theme.conf` alongside colours; named colour-only themes inherit the user's placement/margin unchanged.
- **Click-outside-to-close** — the layer-shell surface now covers the full monitor (transparent background). Clicks inside the popup box are absorbed; clicks anywhere outside it close the popup. This matches the compositor interaction model used by trebuchet.

### Removed
- `veu.conf` (separate placement config) — placement and margin have moved into `theme.conf`; the `config` module has been eliminated.

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
