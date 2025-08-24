use crate::screen::{Location, Mode, Output, Resolution, Screen};
use regex::Regex;

struct Parser {
    output_line_regex: Regex,
    mode_line_regex: Regex,
    freq_regex: Regex,
}

impl Parser {
    fn new() -> Self {
        Self {
            output_line_regex: Regex::new(
                r"(?x)
                ^(?P<name>\S+)
                \s(?P<status>connected|disconnected)
                (?:\sprimary)?
                (?:\s(?P<resolution>\d+x\d+\+\d+\+\d+))?
                \s
            ",
            )
            .expect("bad output_line_regex"),
            mode_line_regex: Regex::new(
                r"^\s+(?P<width>\d+)x(?P<height>\d+)(?P<freqs>(?:\s+\d+\.\d{2}[ *][ +])+)$",
            )
            .expect("bad mode_line_regex"),
            freq_regex: Regex::new(r"(\d+)\.(\d{2})").expect("bad freq_regex"),
        }
    }

    fn parse_output_line(&self, line: &str) -> Option<Output> {
        self.output_line_regex.captures(line).map(|caps| Output {
            name: caps["name"].to_string(),
            connected: &caps["status"] == "connected",
            enabled: caps.name("resolution").is_some(),
            modes: Vec::new(),
            location: Location::from_output_name(&caps["name"]),
        })
    }

    fn parse_mode_line(&self, line: &str, modes: &mut Vec<Mode>) {
        let Some(caps) = self.mode_line_regex.captures(line) else {
            return;
        };

        let resolution = Resolution {
            width: caps["width"].parse().expect("bad width"),
            height: caps["height"].parse().expect("bad height"),
        };

        for caps in self.freq_regex.captures_iter(&caps["freqs"]) {
            let x: u32 = caps[1].parse().expect("bad integer part");
            let y: u32 = caps[2].parse().expect("bad fractional part");
            assert!((0..100).contains(&y));
            let refresh_rate = x * 1000 + y * 10;

            modes.push(Mode {
                resolution,
                refresh_rate,
            });
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
                self.parse_mode_line(line, &mut output.modes);
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
    fn parse_output_line_must_return_nothing() {
        let parser = Parser::new();
        assert!(parser.parse_output_line(SCREEN_LINE).is_none());
        assert!(
            parser
                .parse_output_line(ACTIVE_PREFERRED_MODE_LINE)
                .is_none()
        );
        assert!(parser.parse_output_line(ACTIVE_MODE_LINE).is_none());
        assert!(parser.parse_output_line(PREFERRED_MODE_LINE).is_none());
        assert!(parser.parse_output_line(PLAIN_MODE_LINE).is_none());
        for line in VERBOSE_INFO_LINES {
            assert!(parser.parse_output_line(line).is_none());
        }
    }

    #[test]
    fn parse_output_line_must_return_something() {
        // Arrange
        struct TestCase {
            line: &'static str,
            name: &'static str,
            connected: bool,
            enabled: bool,
            location: Location,
        }

        let test_cases = [
            TestCase {
                line: CONNECTED_ENABLED_INTERNAL_OUTPUT_LINE,
                name: "eDP-1",
                connected: true,
                enabled: true,
                location: Location::Internal,
            },
            TestCase {
                line: CONNECTED_DISABLED_EXTERNAL_OUTPUT_LINE,
                name: "HDMI-2",
                connected: true,
                enabled: false,
                location: Location::External,
            },
            TestCase {
                line: DISCONNECTED_ENABLED_EXTERNAL_OUTPUT_LINE,
                name: "HDMI-2",
                connected: false,
                enabled: true,
                location: Location::External,
            },
            TestCase {
                line: DISCONNECTED_DISABLED_EXTERNAL_OUTPUT_LINE,
                name: "DP-1",
                connected: false,
                enabled: false,
                location: Location::External,
            },
        ];

        let parser = Parser::new();

        for test_case in test_cases {
            // Act
            let output = parser.parse_output_line(test_case.line);

            // Assert
            let Some(output) = output else {
                panic!("expected some output");
            };
            assert_eq!(output.name, test_case.name);
            assert_eq!(output.connected, test_case.connected);
            assert_eq!(output.enabled, test_case.enabled);
            assert_eq!(output.location, test_case.location);
        }
    }

    #[test]
    fn parse_mode_line_must_ignore_non_mode_lines() {
        // Arrange
        let mut modes = Vec::new();
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(SCREEN_LINE, &mut modes);
        parser.parse_mode_line(CONNECTED_ENABLED_INTERNAL_OUTPUT_LINE, &mut modes);
        parser.parse_mode_line(CONNECTED_DISABLED_EXTERNAL_OUTPUT_LINE, &mut modes);
        parser.parse_mode_line(DISCONNECTED_ENABLED_EXTERNAL_OUTPUT_LINE, &mut modes);
        parser.parse_mode_line(DISCONNECTED_DISABLED_EXTERNAL_OUTPUT_LINE, &mut modes);
        for line in VERBOSE_INFO_LINES {
            parser.parse_mode_line(line, &mut modes);
        }

        // Assert
        assert!(modes.is_empty());
    }

    #[test]
    fn parse_mode_line_must_parse_active_preferred_mode_line() {
        // Arrange
        let mut modes = Vec::new();
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(ACTIVE_PREFERRED_MODE_LINE, &mut modes);

        // Assert
        assert_eq!(
            modes,
            [
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 60020,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 60010,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 59970,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 59960,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 59930,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 48020,
                },
            ]
        );
    }

    #[test]
    fn parse_mode_line_must_parse_active_mode_line() {
        // Arrange
        let mut modes = Vec::new();
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(ACTIVE_MODE_LINE, &mut modes);

        // Assert
        assert_eq!(
            modes,
            [
                Mode {
                    resolution: Resolution {
                        width: 1680,
                        height: 1050
                    },
                    refresh_rate: 59950,
                },
                Mode {
                    resolution: Resolution {
                        width: 1680,
                        height: 1050
                    },
                    refresh_rate: 59880,
                },
            ]
        );
    }

    #[test]
    fn parse_mode_line_must_parse_preferred_mode_line() {
        // Arrange
        let mut modes = Vec::new();
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(PREFERRED_MODE_LINE, &mut modes);

        // Assert
        assert_eq!(
            modes,
            [
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 60020,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 60010,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 59970,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 59960,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 59930,
                },
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080
                    },
                    refresh_rate: 48020,
                },
            ]
        );
    }
    #[test]
    fn parse_mode_line_must_parse_plain_mode_line() {
        // Arrange
        let mut modes = Vec::new();
        let parser = Parser::new();

        // Act
        parser.parse_mode_line(PLAIN_MODE_LINE, &mut modes);

        // Assert
        assert_eq!(
            modes,
            [
                Mode {
                    resolution: Resolution {
                        width: 1680,
                        height: 1050
                    },
                    refresh_rate: 59950,
                },
                Mode {
                    resolution: Resolution {
                        width: 1680,
                        height: 1050
                    },
                    refresh_rate: 59880,
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
        assert!(screen.outputs[2].enabled);
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

    const CONNECTED_ENABLED_INTERNAL_OUTPUT_LINE: &str = "eDP-1 connected primary 1920x1080+0+0 (normal left inverted right x axis y axis) 344mm x 194mm";
    const CONNECTED_DISABLED_EXTERNAL_OUTPUT_LINE: &str =
        "HDMI-2 connected (normal left inverted right x axis y axis)";
    const DISCONNECTED_ENABLED_EXTERNAL_OUTPUT_LINE: &str =
        "HDMI-2 disconnected 1920x1080+0+0 (normal left inverted right x axis y axis) 0mm x 0mm";
    const DISCONNECTED_DISABLED_EXTERNAL_OUTPUT_LINE: &str =
        "DP-1 disconnected (normal left inverted right x axis y axis)";

    const ACTIVE_PREFERRED_MODE_LINE: &str =
        "   1920x1080     60.02*+  60.01    59.97    59.96    59.93    48.02  ";
    const ACTIVE_MODE_LINE: &str = "   1680x1050     59.95*   59.88  ";
    const PREFERRED_MODE_LINE: &str =
        "   1920x1080     60.02 +  60.01    59.97    59.96    59.93    48.02  ";
    const PLAIN_MODE_LINE: &str = "   1680x1050     59.95    59.88  ";
    const VERBOSE_INFO_LINES: [&str; 3] = [
        "  1920x1080 (0x501) 148.500MHz +HSync +VSync ",
        "        h: width  1920 start 2008 end 2052 total 2200 skew    0 clock  67.50KHz ",
        "        v: height 1080 start 1084 end 1089 total 1125           clock  60.00Hz ",
    ];

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
HDMI-1 disconnected 1920x1080+0+0 (normal left inverted right x axis y axis) 0mm x 0mm
  1920x1080 (0x501) 148.500MHz +HSync +VSync
        h: width  1920 start 2008 end 2052 total 2200 skew    0 clock  67.50KHz
        v: height 1080 start 1084 end 1089 total 1125           clock  60.00Hz
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
