use crate::screen::{Location, Mode, Output, Resolution, Screen};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RpcOutput<'a> {
    name: &'a str,
    active: bool,
    modes: Vec<RpcMode>,
}

#[derive(Debug, Deserialize)]
struct RpcMode {
    width: i32,
    height: i32,
    refresh: i32,
}

pub(super) fn parse(swaymsg_output: &[u8]) -> Screen {
    let rpc_outputs: Vec<RpcOutput> = serde_json::from_slice(swaymsg_output)
        .expect("failed to parse output of swaymsg -t get_outputs");

    Screen {
        outputs: rpc_outputs
            .iter()
            .map(|rpc_output| Output {
                name: rpc_output.name.to_string(),
                // Sway does not return disconnected outputs
                connected: true,
                enabled: rpc_output.active,
                modes: rpc_output
                    .modes
                    .iter()
                    .map(|rpc_mode| Mode {
                        resolution: Resolution {
                            width: rpc_mode.width,
                            height: rpc_mode.height,
                        },
                        refresh_rate: rpc_mode.refresh,
                    })
                    .collect(),
                location: Location::from_output_name(rpc_output.name),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_outputs_output_parses_ok() {
        // Arrange

        // Act
        let screen = parse(TEST_GET_OUTPUTS.as_bytes());

        // Assert
        assert_eq!(screen.outputs.len(), 2);
        assert_eq!(screen.outputs[0].name, "HDMI-A-2");
        assert!(screen.outputs[0].connected);
        assert!(screen.outputs[0].enabled);
        assert_eq!(screen.outputs[0].modes.len(), 35);
        assert_eq!(
            screen.outputs[0].modes[0],
            Mode {
                resolution: Resolution {
                    width: 4096,
                    height: 2160
                },
                refresh_rate: 30000
            }
        );
        assert_eq!(screen.outputs[1].name, "eDP-1");
        assert!(screen.outputs[1].connected);
        assert!(!screen.outputs[1].enabled);
        assert_eq!(screen.outputs[1].modes.len(), 2);
    }

    const TEST_GET_OUTPUTS: &str = r#"
[
  {
    "id": 4,
    "type": "output",
    "orientation": "none",
    "percent": 1.0,
    "urgent": false,
    "marks": [],
    "layout": "output",
    "border": "none",
    "current_border_width": 0,
    "rect": {
      "x": 0,
      "y": 0,
      "width": 1536,
      "height": 864
    },
    "deco_rect": {
      "x": 0,
      "y": 0,
      "width": 0,
      "height": 0
    },
    "window_rect": {
      "x": 0,
      "y": 0,
      "width": 0,
      "height": 0
    },
    "geometry": {
      "x": 0,
      "y": 0,
      "width": 0,
      "height": 0
    },
    "name": "HDMI-A-2",
    "window": null,
    "nodes": [],
    "floating_nodes": [],
    "focus": [
      6
    ],
    "fullscreen_mode": 0,
    "sticky": false,
    "floating": null,
    "scratchpad_state": null,
    "primary": false,
    "make": "Shenzhen KTC Technology Group",
    "model": "49'TV",
    "serial": "0x00000001",
    "modes": [
      {
        "width": 4096,
        "height": 2160,
        "refresh": 30000,
        "picture_aspect_ratio": "256:135"
      },
      {
        "width": 4096,
        "height": 2160,
        "refresh": 29970,
        "picture_aspect_ratio": "256:135"
      },
      {
        "width": 4096,
        "height": 2160,
        "refresh": 25000,
        "picture_aspect_ratio": "256:135"
      },
      {
        "width": 4096,
        "height": 2160,
        "refresh": 24000,
        "picture_aspect_ratio": "256:135"
      },
      {
        "width": 4096,
        "height": 2160,
        "refresh": 23976,
        "picture_aspect_ratio": "256:135"
      },
      {
        "width": 3840,
        "height": 2160,
        "refresh": 30000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 3840,
        "height": 2160,
        "refresh": 29970,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 3840,
        "height": 2160,
        "refresh": 25000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 3840,
        "height": 2160,
        "refresh": 24000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 3840,
        "height": 2160,
        "refresh": 23976,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 60000,
        "picture_aspect_ratio": "none"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 60000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 59940,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 50000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 30000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 29970,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 25000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 24000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 23976,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1600,
        "height": 900,
        "refresh": 60000,
        "picture_aspect_ratio": "none"
      },
      {
        "width": 1280,
        "height": 1024,
        "refresh": 60020,
        "picture_aspect_ratio": "none"
      },
      {
        "width": 1280,
        "height": 720,
        "refresh": 60000,
        "picture_aspect_ratio": "none"
      },
      {
        "width": 1280,
        "height": 720,
        "refresh": 60000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1280,
        "height": 720,
        "refresh": 59940,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1280,
        "height": 720,
        "refresh": 50000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 1024,
        "height": 768,
        "refresh": 60004,
        "picture_aspect_ratio": "none"
      },
      {
        "width": 800,
        "height": 600,
        "refresh": 60317,
        "picture_aspect_ratio": "none"
      },
      {
        "width": 720,
        "height": 576,
        "refresh": 50000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 720,
        "height": 576,
        "refresh": 50000,
        "picture_aspect_ratio": "4:3"
      },
      {
        "width": 720,
        "height": 480,
        "refresh": 60000,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 720,
        "height": 480,
        "refresh": 60000,
        "picture_aspect_ratio": "4:3"
      },
      {
        "width": 720,
        "height": 480,
        "refresh": 59940,
        "picture_aspect_ratio": "16:9"
      },
      {
        "width": 720,
        "height": 480,
        "refresh": 59940,
        "picture_aspect_ratio": "4:3"
      },
      {
        "width": 640,
        "height": 480,
        "refresh": 60000,
        "picture_aspect_ratio": "4:3"
      },
      {
        "width": 640,
        "height": 480,
        "refresh": 59940,
        "picture_aspect_ratio": "none"
      }
    ],
    "features": {
      "adaptive_sync": false,
      "hdr": false
    },
    "non_desktop": false,
    "active": true,
    "dpms": true,
    "power": true,
    "scale": 1.25,
    "scale_filter": "linear",
    "transform": "normal",
    "adaptive_sync_status": "disabled",
    "current_workspace": "2",
    "current_mode": {
      "width": 1920,
      "height": 1080,
      "refresh": 60000,
      "picture_aspect_ratio": "none"
    },
    "max_render_time": 0,
    "allow_tearing": false,
    "hdr": false,
    "focused": true,
    "subpixel_hinting": "unknown"
  },
  {
    "primary": false,
    "make": "Lenovo Group Limited",
    "model": "0x40BA",
    "serial": "Unknown",
    "modes": [
      {
        "width": 1920,
        "height": 1080,
        "refresh": 60020
      },
      {
        "width": 1920,
        "height": 1080,
        "refresh": 48016
      }
    ],
    "features": {
      "adaptive_sync": false,
      "hdr": false
    },
    "non_desktop": false,
    "type": "output",
    "name": "eDP-1",
    "active": false,
    "dpms": false,
    "power": false,
    "current_workspace": null,
    "rect": {
      "x": 0,
      "y": 0,
      "width": 0,
      "height": 0
    },
    "percent": null
  }
]
    "#;
}
