# veu

Wayland layer-shell popup for quickly controlling PipeWire audio volume.

Built with [iced](https://github.com/iced-rs/iced) and [iced-layershell](https://github.com/waycrate/exwlshelleventloop).

<img width="200" height="200" alt="icon_no_bg" src="https://github.com/user-attachments/assets/2f774ae7-121c-46e1-acfa-a1c40fae2eec" />


## Features

- Output and input volume sliders (0–150%)
- Mute All toggle
- Closes on Escape or click outside
- All PipeWire interaction via `wpctl`

## Requirements

- Wayland compositor with `wlr-layer-shell` support (Hyprland, Sway, etc.)
- PipeWire + WirePlumber (`wpctl` in PATH)
- Rust toolchain (for building from source)

## Installation

**System-wide** (installs to `/usr/local/bin`, requires sudo):

```sh
bash scripts/install.sh
```

**Current user only** (installs to `~/.local/bin`, no sudo):

```sh
bash scripts/install.sh --user
```

**One-line install from GitHub:**

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/rafaelzimmermann/veu/main/scripts/install.sh)"
```

### Uninstall

```sh
bash scripts/install.sh --uninstall
```

## Usage

Bind veu to a key in your compositor config, e.g. Hyprland:

```
bind = $mod, V, exec, veu
```

### Theming

The active theme is read from `~/.config/veu/theme.conf` on each launch.
Switch to a bundled theme by writing its name to `~/.config/veu/current-theme`:

```sh
echo catppuccin-mocha > ~/.config/veu/current-theme
```

Bundled themes: `default`, `catppuccin-mocha`, `dracula`, `gruvbox-dark`, `nord`, `tokyo-night`.

To customise, edit `~/.config/veu/theme.conf` (installed automatically, or copy from `assets/theme.conf`).

## Project layout

```
src/
├── main.rs                  # entry point, layer-shell window settings
├── app/
│   ├── mod.rs               # app state, message routing, outer container
│   └── components/
│       ├── mod.rs
│       └── volume.rs        # volume control UI component
├── audio/
│   └── mod.rs               # PipeWire abstraction (load, set, mute via wpctl)
└── theme/
    └── mod.rs               # Theme struct, load from ~/.config/veu/theme.conf
```
