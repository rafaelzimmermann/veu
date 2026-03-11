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
- Rust toolchain

## Build

```sh
cargo build --release
```

Binary at `target/release/veu`.

## Usage

```sh
./veu
```

Bind it to a key in your compositor config, e.g. Hyprland:

```
bind = $mod, V, exec, /path/to/veu
```

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
    └── mod.rs               # shared colour constants
```
