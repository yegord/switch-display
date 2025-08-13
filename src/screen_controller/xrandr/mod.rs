use super::utils::run;
use crate::screen::{Resolution, Screen};
use crate::switch::SwitchPlan;
use std::process;

mod parsing;

struct Xrandr {
    command: process::Command,
}

impl Xrandr {
    fn new() -> Self {
        let command = process::Command::new("xrandr");
        Self { command }
    }

    fn output(mut self, output_name: &str) -> Self {
        self.command.arg("--output").arg(output_name);
        self
    }

    fn mode(mut self, resolution: Option<Resolution>) -> Self {
        if let Some(resolution) = resolution {
            self.command
                .arg("--mode")
                .arg(format!("{}x{}", resolution.width, resolution.height));
        } else {
            self.command.arg("--auto");
        }
        self
    }

    fn same_as(mut self, output_name: &str) -> Self {
        self.command.arg("--same-as").arg(output_name);
        self
    }

    fn off(mut self) -> Self {
        self.command.arg("--off");
        self
    }

    fn command(self) -> process::Command {
        self.command
    }
}

pub(super) fn get_outputs() -> Screen {
    let status = run(Xrandr::new().command());
    let xrandr_output = String::from_utf8(status.stdout).expect("xrandr output is invalid utf-8");
    parsing::parse(&xrandr_output)
}

fn build_switch_commands(
    switch_plan: &SwitchPlan,
    resolution: Option<Resolution>,
) -> Vec<process::Command> {
    let disable_commands = switch_plan
        .outputs_to_disable
        .iter()
        .map(|output| Xrandr::new().output(&output.name).off().command());

    let enable_commands = switch_plan
        .outputs_to_enable
        .split_first()
        .map(|(first, other)| {
            let first_command = Xrandr::new().output(&first.name).mode(resolution).command();

            let other_commands = other.iter().map(|output| {
                Xrandr::new()
                    .output(&output.name)
                    .mode(resolution)
                    .same_as(&first.name)
                    .command()
            });

            std::iter::once(first_command).chain(other_commands)
        })
        .into_iter()
        .flatten();

    disable_commands.chain(enable_commands).collect()
}

pub(super) fn switch_outputs(switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
    for command in build_switch_commands(switch_plan, resolution) {
        run(command);
    }
}

#[cfg(test)]
mod tests {
    use super::super::utils::assert_command_eq;
    use super::*;
    use crate::screen::{Location, Output};

    #[test]
    fn test_make_switch_commands_without_resolution() {
        // Arrange
        let outputs = [
            Output {
                name: "eDP-1".to_string(),
                connected: true,
                enabled: true,
                modes: Vec::new(),
                location: Location::Internal,
            },
            Output {
                name: "HDMI-1".to_string(),
                connected: true,
                enabled: false,
                modes: Vec::new(),
                location: Location::External,
            },
            Output {
                name: "HDMI-2".to_string(),
                connected: false,
                enabled: true,
                modes: Vec::new(),
                location: Location::External,
            },
        ];

        let switch_plan = SwitchPlan {
            outputs_to_disable: vec![&outputs[2]],
            outputs_to_enable: vec![&outputs[0], &outputs[1]],
        };

        let resolution = None;

        // Act
        let commands = build_switch_commands(&switch_plan, resolution);

        // Assert
        assert!(commands.len() == 3);
        assert_command_eq(&commands[0], "xrandr", &["--output", "HDMI-2", "--off"]);
        assert_command_eq(&commands[1], "xrandr", &["--output", "eDP-1", "--auto"]);
        assert_command_eq(
            &commands[2],
            "xrandr",
            &["--output", "HDMI-1", "--auto", "--same-as", "eDP-1"],
        );
    }

    #[test]
    fn test_make_switch_commands_with_resolution() {
        // Arrange
        let outputs = [
            Output {
                name: "eDP-1".to_string(),
                connected: true,
                enabled: true,
                modes: Vec::new(),
                location: Location::Internal,
            },
            Output {
                name: "HDMI-1".to_string(),
                connected: true,
                enabled: false,
                modes: Vec::new(),
                location: Location::External,
            },
            Output {
                name: "HDMI-2".to_string(),
                connected: false,
                enabled: true,
                modes: Vec::new(),
                location: Location::External,
            },
        ];

        let switch_plan = SwitchPlan {
            outputs_to_disable: vec![&outputs[2]],
            outputs_to_enable: vec![&outputs[0], &outputs[1]],
        };

        let resolution = Some(Resolution {
            width: 1920,
            height: 1080,
        });

        // Act
        let commands = build_switch_commands(&switch_plan, resolution);

        // Assert
        assert!(commands.len() == 3);
        assert_command_eq(&commands[0], "xrandr", &["--output", "HDMI-2", "--off"]);
        assert_command_eq(
            &commands[1],
            "xrandr",
            &["--output", "eDP-1", "--mode", "1920x1080"],
        );
        assert_command_eq(
            &commands[2],
            "xrandr",
            &[
                "--output",
                "HDMI-1",
                "--mode",
                "1920x1080",
                "--same-as",
                "eDP-1",
            ],
        );
    }
}
