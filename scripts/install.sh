#!/usr/bin/env bash
# Install veu — Wayland PipeWire volume popup
#
# One-line install (recommended):
#   sh -c "$(curl -fsSL https://raw.githubusercontent.com/spikelynch/veu/main/scripts/install.sh)"
#
# Options:
#   --user        install to ~/.local/bin instead of /usr/local/bin
#   --uninstall   remove installed files
#   --yes         assume yes for all prompts (non-interactive)
#   --no          assume no for all prompts (non-interactive)
#
# When run from the project root (where Cargo.toml lives), the local source
# is used. Otherwise the repository is cloned automatically.

set -euo pipefail

# ── Argument parsing ──────────────────────────────────────────────────────────

SYSTEM=true
UNINSTALL=false
YES=false
NO=false

for arg in "$@"; do
    case "$arg" in
        --user)      SYSTEM=false ;;
        --system)    SYSTEM=true ;;
        --uninstall) UNINSTALL=true ;;
        --yes|-y)    YES=true ;;
        --no|-n)     NO=true ;;
        --help|-h)
            sed -n '2,14p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Run with --help to see available options."
            exit 1
            ;;
    esac
done

# ── Paths ─────────────────────────────────────────────────────────────────────

if $SYSTEM; then
    BIN_DIR="/usr/local/bin"
    DESKTOP_DIR="/usr/local/share/applications"
else
    BIN_DIR="${HOME}/.local/bin"
    DESKTOP_DIR="${HOME}/.local/share/applications"
fi

BINARY="$BIN_DIR/veu"
DESKTOP_FILE="$DESKTOP_DIR/veu.desktop"
CONFIG_DIR="${HOME}/.config/veu"
THEME_FILE="$CONFIG_DIR/theme.conf"
THEMES_DIR="$CONFIG_DIR/themes"

# ── Privilege helpers ─────────────────────────────────────────────────────────

if $SYSTEM && [[ $EUID -ne 0 ]]; then
    PRIV="sudo"
    echo "sudo access is required for system-wide install."
    sudo -v
    ( while true; do sudo -n true; sleep 50; done ) &
    SUDO_KEEPALIVE_PID=$!
    trap 'kill "$SUDO_KEEPALIVE_PID" 2>/dev/null' EXIT
else
    PRIV=""
fi

priv_mkdir()   { $PRIV mkdir -p "$@"; }
priv_install() { $PRIV install "$@"; }
priv_tee()     { $PRIV tee "$@" >/dev/null; }
priv_rm()      { $PRIV rm "$@"; }

confirm() {
    if $YES; then return 0; fi
    if $NO;  then return 1; fi
    local reply
    read -r -n 1 -p "$1 [y/N] " reply
    echo ""
    [[ "${reply,,}" == "y" ]]
}

# ── Uninstall ─────────────────────────────────────────────────────────────────

if $UNINSTALL; then
    echo "Uninstalling veu…"
    priv_rm -f  "$BINARY"
    priv_rm -f  "$DESKTOP_FILE"
    rm -rf "$CONFIG_DIR"
    echo "Done."
    exit 0
fi

# ── Source (clone if not in project root) ─────────────────────────────────────

if [[ ! -f Cargo.toml ]]; then
    if ! command -v git &>/dev/null; then
        echo "Error: git is required to install veu." >&2
        exit 1
    fi
    CLONE_DIR=$(mktemp -d)
    trap 'rm -rf "$CLONE_DIR"' EXIT
    echo "Cloning veu…"
    git clone --depth=1 https://github.com/spikelynch/veu.git "$CLONE_DIR"
    cd "$CLONE_DIR"
fi

# ── Upfront questions ─────────────────────────────────────────────────────────

mkdir -p "$CONFIG_DIR"

OVERWRITE_THEME=false
if [[ ! -f "$THEME_FILE" ]]; then
    : # fresh install — will write theme.conf below
elif confirm "Theme config already exists at $THEME_FILE. Overwrite?"; then
    OVERWRITE_THEME=true
fi

UPDATE_THEMES=false
if [[ ! -d "$THEMES_DIR" || -z "$(ls -A "$THEMES_DIR" 2>/dev/null)" ]]; then
    UPDATE_THEMES=true
elif $OVERWRITE_THEME; then
    UPDATE_THEMES=true
elif confirm "Themes already exist at $THEMES_DIR. Update them?"; then
    UPDATE_THEMES=true
fi

echo ""

# ── Build ──────────────────────────────────────────────────────────────────────

echo "Building veu (release)…"
cargo build --release

# ── Install binary ────────────────────────────────────────────────────────────

echo "Installing to $BIN_DIR…"
priv_mkdir "$BIN_DIR"
priv_install -m 755 target/release/veu "$BINARY"

# ── Config ────────────────────────────────────────────────────────────────────

if [[ ! -f "$THEME_FILE" ]] || $OVERWRITE_THEME; then
    cp assets/theme.conf "$THEME_FILE"
    if $OVERWRITE_THEME; then
        echo "Theme config replaced."
    else
        echo "Default theme config installed to $THEME_FILE."
    fi
else
    echo "Keeping existing theme config."
fi

if $UPDATE_THEMES && [[ -d assets/themes ]]; then
    echo "Installing themes to $THEMES_DIR…"
    mkdir -p "$THEMES_DIR"
    cp assets/themes/*.conf "$THEMES_DIR/"
else
    echo "Keeping existing themes."
fi

# ── Desktop entry ─────────────────────────────────────────────────────────────

priv_mkdir "$DESKTOP_DIR"
priv_tee "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=veu
Comment=Wayland PipeWire volume popup
Exec=veu
Categories=Utility;
NoDisplay=true
EOF

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Installed:  $BINARY"
echo "Desktop:    $DESKTOP_FILE"
echo "Config:     $THEME_FILE"
echo "Themes:     $THEMES_DIR"
echo ""

if ! command -v veu &>/dev/null 2>&1; then
    echo "Note: $BIN_DIR is not on your PATH."
    echo "Add it to your shell profile:"
    echo "  export PATH=\"\$PATH:$BIN_DIR\""
    echo ""
fi

echo "Bind it to a key in your Hyprland config:"
echo "  bind = \$mod, V, exec, veu"
echo ""
echo "Switch themes by writing a theme name to ~/.config/veu/current-theme:"
echo "  echo catppuccin-mocha > ~/.config/veu/current-theme"
