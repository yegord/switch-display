# switch-display

A utility to toggle connected displays between three states in X11 and Sway/Wayland:
1. All connected displays enabled
2. Only external display(s) enabled
3. Only internal display(s) enabled

All enabled displays mirror the same content.
The tool automatically selects the best common resolution supported by all enabled displays.

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
switch-display --controller xrandr  # for X11
switch-display --controller sway    # for Sway/Wayland
```

Request a resolution with a minimum refresh rate (e.g. 50 Hz):
```bash
switch-display --controller xrandr --min-refresh-rate 50000
```

## Integration with window managers

Add keybindings to your config (`~/.config/sway/config` or `~/.config/i3/config`):
```
# Sway
bindsym --locked XF86Display exec exec switch-display --controller sway

# i3
bindsym XF86Display exec --no-startup-id exec switch-display --controller xrandr
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
1. Determines the current state of video outputs
2. Determines the next desired state and the outputs that need to be enabled and disabled to reach this state
3. Finds the best common resolution supported by all displays to be enabled
4. Disables and enables outputs as necessary

The resolution is chosen based on:
1. Highest supported common resolution
2. The minimum required refresh rate (if specified)

If no common resolution is found, a preferred (xrandr) or last/default (sway) one is used.

## License

MIT. See full text in [LICENSE](LICENSE).
