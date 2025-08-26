# switch-display

`switch-display` is a small utility to quickly toggle between display configurations in X11 and Sway/Wayland.
It supports three modes:
- all connected displays enabled,
- only external display(s) enabled,
- only internal display(s) enabled.

When multiple displays are enabled, they mirror the same content.
The tool automatically picks the best common resolution supported by all enabled displays.

The tool can control displays using three different backends ("controllers"):
* `xrandr` (X11), using `xrandr` command-line tool,
* `randr` (X11), using the RandR protocol directly,
* `sway` (Wayland), using `swaymsg` command-line tool.

## Installation

### Prerequisites

You need Rust and Cargo installed.
If you do not have them:

```bash
# Either use rustup (recommended):
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or your distribution's package manager (may be old/outdated):
# For Debian/Ubuntu:
sudo apt install cargo
# For Arch Linux:
sudo pacman -S rust
```

### Install switch-display
```bash
cargo install --git https://github.com/yegord/switch-display
```

## Usage

Basic usage:
```bash
switch-display --controller randr  # for X11
switch-display --controller sway   # for Sway/Wayland
```

Request a display mode with at least 50 Hz refresh rate (value is in millihertz):
```bash
switch-display --controller randr --min-refresh-rate 50000
```

## Integration with window managers

You can bind `switch-display` to the `XF86Display` key (usually present on laptops) or any other key in your window manager config (`~/.config/sway/config` or `~/.config/i3/config`).

Example:

```bash
# Sway
# Note: the double "exec" is intentional.
# - The first "exec" is Sway's command to run a shell.
# - The second "exec" replaces the shell process with switch-display,
#   so no extra forks are done.
bindsym --locked XF86Display exec exec switch-display --controller sway

# i3
# Same idea: using "exec" ensures no intermediate shell process remains.
bindsym XF86Display exec --no-startup-id exec switch-display --controller randr
```

## Troubleshooting

### Command not found

Ensure `~/.cargo/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Verbose logging

```bash
RUST_LOG=trace switch-display --controller sway
```

## How it works

The tool:
1. Detects the current display state
2. Decides the next state (which outputs should be enabled/disabled)
3. Finds the best common resolution that meets the minimum refresh rate
4. Applies the configuration

The highest supported common resolution satisfying the minimum required refresh rate is chosen.

If no common resolution exists, a fallback strategy is used:
* `xrandr` controller will let `xrandr` to decide the exact output mode (`xrandr --output OUTPUT --auto`),
* `randr` controller will try to pick an, ideally, preferred mode according to RandR information, with the largest resolution and the highest frame rate, in the order of decreasing significance,
* `sway` controller will let Sway decide.

## License

MIT. See full text in [LICENSE](LICENSE).
