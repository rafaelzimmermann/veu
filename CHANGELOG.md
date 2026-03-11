# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Settings panel** — gear button (⚙) in the tray header opens a 700 × 520 centred overlay with full audio controls. Closes via ✕ button or click outside.
- **System device selection** — Output and Input pick-lists in the System section set the PipeWire default sink/source via `pactl set-default-sink/source`. Selection is persisted in `~/.config/veu/device-prefs.conf` and re-applied on next launch.
- **Per-application volume and routing** — Applications — Output and Applications — Input sections list all active PipeWire sink-inputs and source-outputs with individual sliders (0–150%) and device pick-lists. Device assignments are stored per app name and restored automatically when settings open.
- **System / Custom routing mode** — each applications section has a segmented pill toggle. *System* mode immediately routes all streams to the current default device and re-applies this at every app startup; *Custom* mode re-enables per-app stored preferences. Mode preference is persisted in `device-prefs.conf`.
- **Startup routing** — `apply_routing_preferences()` runs at boot (before the tray appears) to enforce whichever routing mode was last saved, with no settings panel interaction required.
- **Per-channel mute** — Output and Input each have an individual mute button (the speaker/mic icon). Clicking it toggles that channel via `pactl`/`wpctl`; the icon switches to 🔇 and the entire row dims. Applies in both the tray popup and the settings panel for system and per-app streams.
- **Theme selection in settings** — a THEME pick-list at the bottom of the settings panel lists all installed named themes. Selecting one applies it immediately and persists the choice to `~/.config/veu/current-theme`.
- **Volume percentage readout** — all sliders (tray and settings) display the current value as a `%` label to the right of the handle.

### Changed
- **Tray popup** — gear button moved into the header row alongside Mute All (footer row removed); height reduced from 200 → 180 px; "Mute All" label becomes "Unmute" when active.
- **Settings layout** — section headers (`SYSTEM`, `APPLICATIONS — OUTPUT/INPUT`) rendered in subdued uppercase at 11 px; all rows share aligned fixed-width columns (label · icon · slider · % · dropdown); padding increased to 20 px; spacing tightened throughout.
- **`device-prefs.conf`** — file extended with reserved keys: `__default_sink__`, `__default_source__`, `__sink_input_mode__`, `__source_output_mode__`.
- **`volume.rs` refactor** — view logic split into a `ViewColors` struct (centralises derived colours and button/slider style factories) and a `channel_row()` free function (renders one labelled slider row); eliminates duplication between the Output and Input rows.

### Removed
- Unused `Placement::anchor()`, `Placement::margin()`, and `Theme::from_file()` methods and their associated tests.

### Added
- **Volume feedback sound** — a short system sound plays after releasing the output slider so the user can judge whether the new level is adequate. Uses `paplay` with the freedesktop sound theme (`audio-volume-change.oga`); silently no-ops if `paplay` or the sound file is not available. The sound is applied after `wpctl set-volume` completes so it plays at the new level.
- **Configurable placement** — `placement` and `margin` are now first-class theme fields. The popup can be anchored to any corner or edge (`top-right`, `top-left`, `top-center`, `bottom-right`, `bottom-left`, `bottom-center`, `center`). Both keys live in `theme.conf` alongside colours; named colour-only themes inherit the user's placement/margin unchanged.
- **Click-outside-to-close** — the layer-shell surface now covers the full monitor (transparent background). Clicks inside the popup box are absorbed; clicks anywhere outside it close the popup. This matches the compositor interaction model used by trebuchet.
- **App icon** (`assets/icon.png`) — transparent-background PNG installed to the hicolor icon theme so launchers display it.
- **Launcher visibility** — desktop entry now installs to `/usr/share/applications` (system) or `~/.local/share/applications` (user) with `Icon=veu` and `Categories=AudioVideo;Audio;Utility;`, making veu visible in app launchers. Old misplaced files from prior installs are cleaned up automatically.

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
