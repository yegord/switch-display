use crate::screen::{Location, Mode, Output, Resolution, Screen};
use regex::Regex;

struct Parser {
    output_line_regex: Regex,
    mode_line_regex: Regex,
    freqs_regex: Regex,
}

impl Parser {
    fn new() -> Self {
        Self {
            output_line_regex: Regex::new(r"^(?P<name>\S+)\s+(?P<status>connected|disconnected)")
                .expect("bad output_line_regex"),
            mode_line_regex: Regex::new(r"^\s+(?P<width>\d+)x(?P<height>\d+)\s+(?P<freqs>.*)$")
                .expect("bad mode_line_regex"),
            freqs_regex: Regex::new(r"(?P<x>\d+)\.(?P<y>\d+)(?P<flags>\*?\+?)")
                .expect("bad freqs_regex"),
        }
    }

    fn parse_output_line(&self, line: &str) -> Option<Output> {
        self.output_line_regex.captures(line).map(|caps| Output {
            name: caps["name"].to_string(),
            connected: &caps["status"] == "connected",
            enabled: false,
            modes: Vec::new(),
            location: Location::from_output_name(&caps["name"]),
        })
    }

    fn parse_mode_line(&self, line: &str, output: &mut Output) {
        let Some(caps) = self.mode_line_regex.captures(line) else {
            return;
        };

        let resolution = Resolution {
            width: caps["width"].parse().expect("bad width"),
            height: caps["height"].parse().expect("bad height"),
        };

        for caps in self.freqs_regex.captures_iter(&caps["freqs"]) {
            let x: i32 = caps["x"].parse().expect("bad integer part");
            let y: i32 = caps["y"].parse().expect("bad fractional part");
            assert!(y < 100);
            let freq = x * 1000 + y * 10;

            output.modes.push(Mode { resolution, freq });

            if caps["flags"].contains('*') {
                assert!(!output.enabled);
                output.enabled = true;
            }
        }
    }

    fn parse(&self, xrandr_output: &str) -> Screen {
        let mut outputs = Vec::new();
        let mut current_output: Option<Output> = None;

        for line in xrandr_output.lines() {
            if let Some(output) = self.parse_output_line(line) {
                if let Some(output) = current_output {
                    outputs.push(output);
                }
                current_output = Some(output);
            } else if let Some(output) = current_output.as_mut() {
                self.parse_mode_line(line, output);
            }
        }

        if let Some(output) = current_output {
            outputs.push(output);
        }

        Screen { outputs }
    }
}

pub(super) fn parse(xrandr_output: &str) -> Screen {
    Parser::new().parse(xrandr_output)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_output_line_must_fail() {
        let parser = Parser::new();
        assert!(parser.parse_output_line(SCREEN_LINE).is_none());
        assert!(parser.parse_output_line(ACTIVE_MODE_LINE).is_none());
        assert!(parser.parse_output_line(INACTIVE_MODE_LINE).is_none());
    }

    #[test]
    fn parse_output_line_must_succeed_on_connected_internal_output_line() {
        // Arrange
        let parser = Parser::new();

        // Act
        let output = parser.parse_output_line(CONNECTED_INTERNAL_OUTPUT_LINE);

        // Assert
        let Some(output) = output else {
            panic!("expected some output");
        };
        assert_eq!(output.name, "eDP-1");
        assert!(output.connected);
        assert_eq!(output.location, Location::Internal);
    }

    #[test]
    fn parse_output_line_must_succeed_on_connected_external_output_line() {
        // Arrange
        let parser = Parser::new();

        // Act
        let output = parser.parse_output_line(CONNECTED_EXTERNAL_OUTPUT_LINE);

        // Assert
        let Some(output) = output else {
            panic!("expected some output");
        };
        assert_eq!(output.name, "HDMI-2");
        assert!(output.connected);
        assert_eq!(output.location, Location::External);
    }

    #[test]
    fn parse_output_line_must_succeed_on_disconnected_output_line() {
        // Arrange
        let parser = Parser::new();

        // Act
        let output = parser.parse_output_line(DISCONNECTED_OUTPUT_LINE);

        // Assert
        let Some(output) = output else {
            panic!("expected some output");
        };
        assert_eq!(output.name, "DP-1");
        assert!(!output.connected);
        assert_eq!(output.location, Location::External);
    }

    #[test]
    fn parse_mode_line_must_fail() {
        // Arrange
        let mut output = Output {
            name: "eDP-1".to_string(),
            connected: true,
            enabled: false,
            modes: Vec::new(),
            location: Location::Internal,
        };
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(SCREEN_LINE, &mut output);
        parser.parse_mode_line(CONNECTED_INTERNAL_OUTPUT_LINE, &mut output);
        parser.parse_mode_line(CONNECTED_EXTERNAL_OUTPUT_LINE, &mut output);
        parser.parse_mode_line(DISCONNECTED_OUTPUT_LINE, &mut output);

        // Assert
        assert!(!output.enabled);
        assert!(output.modes.is_empty());
    }

    #[test]
    fn parse_mode_line_must_add_active_modes_and_set_enabled() {
        // Arrange
        let mut output = Output {
            name: "eDP-1".to_string(),
            connected: true,
            enabled: false,
            modes: Vec::new(),
            location: Location::Internal,
        };
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(ACTIVE_MODE_LINE, &mut output);

        // Assert
        assert!(output.enabled);
        assert_eq!(
            output.modes,
            [
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    freq: 60020,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    freq: 60010,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    freq: 59970,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    freq: 59960,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    freq: 59930,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    freq: 48020,
                },
            ]
        );
    }

    #[test]
    fn parse_mode_line_must_add_inactive_modes_and_not_set_enabled() {
        // Arrange
        let mut output = Output {
            name: "eDP-1".to_string(),
            connected: true,
            enabled: false,
            modes: Vec::new(),
            location: Location::Internal,
        };
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(INACTIVE_MODE_LINE, &mut output);

        // Assert
        assert!(!output.enabled);
        assert_eq!(
            output.modes,
            [
                Mode {
                    resolution: Resolution {
                        width: 1680,
                        height: 1050
                    },
                    freq: 59950,
                },
                Mode {
                    resolution: Resolution {
                        width: 1680,
                        height: 1050
                    },
                    freq: 59880,
                },
            ]
        );
    }

    #[test]
    fn test_parse_output() {
        // Arrange
        let parser = Parser::new();

        // Act
        let screen = parser.parse(TEST_OUTPUT);

        // Assert
        assert_eq!(screen.outputs.len(), 5);
        assert_eq!(screen.outputs[0].name, "eDP-1");
        assert!(screen.outputs[0].connected);
        assert!(screen.outputs[0].enabled);
        assert_eq!(screen.outputs[0].modes.len(), 83);
        assert_eq!(screen.outputs[1].name, "DP-1");
        assert!(!screen.outputs[1].connected);
        assert!(!screen.outputs[1].enabled);
        assert!(screen.outputs[1].modes.is_empty());
        assert_eq!(screen.outputs[2].name, "HDMI-1");
        assert!(!screen.outputs[2].connected);
        assert!(!screen.outputs[2].enabled);
        assert!(screen.outputs[2].modes.is_empty());
        assert_eq!(screen.outputs[3].name, "DP-2");
        assert!(!screen.outputs[3].connected);
        assert!(!screen.outputs[3].enabled);
        assert!(screen.outputs[3].modes.is_empty());
        assert_eq!(screen.outputs[4].name, "HDMI-2");
        assert!(screen.outputs[4].connected);
        assert!(!screen.outputs[4].enabled);
        assert_eq!(screen.outputs[4].modes.len(), 30);
    }

    const SCREEN_LINE: &str =
        "Screen 0: minimum 320 x 200, current 1920 x 1080, maximum 16384 x 16384";
    const ACTIVE_MODE_LINE: &str =
        "   1920x1080     60.02*+  60.01    59.97    59.96    59.93    48.02  ";
    const INACTIVE_MODE_LINE: &str = "   1680x1050     59.95    59.88  ";
    const CONNECTED_INTERNAL_OUTPUT_LINE: &str = "eDP-1 connected primary 1920x1080+0+0 (normal left inverted right x axis y axis) 344mm x 194mm";
    const CONNECTED_EXTERNAL_OUTPUT_LINE: &str =
        "HDMI-2 connected (normal left inverted right x axis y axis)";
    const DISCONNECTED_OUTPUT_LINE: &str =
        "DP-1 disconnected (normal left inverted right x axis y axis)";

    const TEST_OUTPUT: &str = r#"
Screen 0: minimum 320 x 200, current 1920 x 1080, maximum 16384 x 16384
eDP-1 connected primary 1920x1080+0+0 (normal left inverted right x axis y axis) 344mm x 194mm
   1920x1080     60.02*+  60.01    59.97    59.96    59.93    48.02  
   1680x1050     59.95    59.88  
   1400x1050     59.98  
   1600x900      59.99    59.94    59.95    59.82  
   1280x1024     60.02  
   1400x900      59.96    59.88  
   1280x960      60.00  
   1440x810      60.00    59.97  
   1368x768      59.88    59.85  
   1280x800      59.99    59.97    59.81    59.91  
   1280x720      60.00    59.99    59.86    59.74  
   1024x768      60.04    60.00  
   960x720       60.00  
   928x696       60.05  
   896x672       60.01  
   1024x576      59.95    59.96    59.90    59.82  
   960x600       59.93    60.00  
   960x540       59.96    59.99    59.63    59.82  
   800x600       60.00    60.32    56.25  
   840x525       60.01    59.88  
   864x486       59.92    59.57  
   700x525       59.98  
   800x450       59.95    59.82  
   640x512       60.02  
   700x450       59.96    59.88  
   640x480       60.00    59.94  
   720x405       59.51    58.99  
   684x384       59.88    59.85  
   640x400       59.88    59.98  
   640x360       59.86    59.83    59.84    59.32  
   512x384       60.00  
   512x288       60.00    59.92  
   480x270       59.63    59.82  
   400x300       60.32    56.34  
   432x243       59.92    59.57  
   320x240       60.05  
   360x202       59.51    59.13  
   320x180       59.84    59.32  
DP-1 disconnected (normal left inverted right x axis y axis)
HDMI-1 disconnected (normal left inverted right x axis y axis)
DP-2 disconnected (normal left inverted right x axis y axis)
HDMI-2 connected (normal left inverted right x axis y axis)
   4096x2160     30.00    25.00    24.00    29.97    23.98  
   3840x2160     30.00    25.00    24.00    29.97    23.98  
   1920x1080     60.00    50.00    59.94    30.00    25.00    24.00    29.97    23.98  
   1920x1080i    60.00    50.00    59.94  
   1600x900      60.00  
   1280x1024     60.02  
   1280x720      60.00    50.00    59.94  
   1024x768      60.00  
   800x600       60.32  
   720x576       50.00  
   720x576i      50.00  
   720x480       60.00    59.94  
   720x480i      60.00    59.94  
   640x480       60.00    59.94  
"#;
}

