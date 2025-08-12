use crate::screen::{Location, Output, Screen};

fn parse_output_line(line: &str) -> Option<Output> {
    if line.contains("connected") {
        let name = line
            .split_ascii_whitespace()
            .next()
            .expect("expected output name here");

        Some(Output {
            name: name.to_string(),
            connected: !line.contains("disconnected"),
            enabled: false,
            modes: Vec::new(),
            location: Location::from_output_name(name),
        })
    } else {
        None
    }
}

fn parse_output(xrandr_output: &str) -> Screen {
    let mut outputs = Vec::new();
    let mut current_output: Option<Output> = None;

    for line in xrandr_output.lines() {
        if let Some(output) = parse_output_line(line) {
            if let Some(output) = current_output {
                outputs.push(output);
            }
            current_output = Some(output);
        }
    }

    if let Some(output) = current_output {
        outputs.push(output);
    }

    Screen { outputs }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_output_line_must_fail() {
        assert!(parse_output_line(SCREEN_LINE).is_none());
        assert!(parse_output_line(ACTIVE_MODE_LINE).is_none());
        assert!(parse_output_line(INACTIVE_MODE_LINE).is_none());
    }

    #[test]
    fn parse_output_line_must_succeed_on_connected_internal_output_line() {
        // Arrange

        // Act
        let output = parse_output_line(CONNECTED_INTERNAL_OUTPUT_LINE);

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

        // Act
        let output = parse_output_line(CONNECTED_EXTERNAL_OUTPUT_LINE);

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

        // Act
        let output = parse_output_line(DISCONNECTED_OUTPUT_LINE);

        // Assert
        let Some(output) = output else {
            panic!("expected some output");
        };
        assert_eq!(output.name, "DP-1");
        assert!(!output.connected);
        assert_eq!(output.location, Location::External);
    }

    #[test]
    fn test_parse_output() {
        // Arrange

        // Act
        let screen = parse_output(TEST_OUTPUT);

        // Assert
        assert_eq!(screen.outputs.len(), 5);
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
