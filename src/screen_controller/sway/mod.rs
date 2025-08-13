use std::fmt::Write;
use std::process;

use crate::{
    screen::{Resolution, Screen},
    switch::SwitchPlan,
};

use super::utils::run;

mod parsing;

struct Swaymsg {
    command: process::Command,
}

impl Swaymsg {
    fn new() -> Self {
        Self {
            command: process::Command::new("swaymsg"),
        }
    }
    fn get_outputs(mut self) -> Self {
        self.command.arg("-t").arg("get_outputs");
        self
    }

    fn disable(mut self, output_name: &str) -> Self {
        self.command
            .arg(format!("output \"{output_name}\" disable"));
        self
    }

    fn enable(mut self, output_name: &str, resolution: Option<Resolution>) -> Self {
        let mut msg = format!("output \"{output_name}\" enable position 0 0");
        if let Some(resolution) = resolution {
            write!(
                &mut msg,
                " mode \"{}x{}\"",
                resolution.width, resolution.height
            ).expect("unable to append to msg");
        }
        self.command.arg(msg);
        self
    }

    fn command(self) -> process::Command {
        self.command
    }
}

pub(super) fn get_outputs() -> Screen {
    parsing::parse(&run(Swaymsg::new().get_outputs().command()).stdout)
}

fn build_switch_commands(
    switch_plan: &SwitchPlan,
    resolution: Option<Resolution>,
) -> Vec<process::Command> {
    let disable_commands = switch_plan
        .outputs_to_disable
        .iter()
        .map(|output| Swaymsg::new().disable(&output.name).command());

    let enable_commands = switch_plan
        .outputs_to_enable
        .iter()
        .map(|output| Swaymsg::new().enable(&output.name, resolution).command());

    disable_commands.chain(enable_commands).collect()
}

pub(super) fn switch_outputs(switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
    for command in build_switch_commands(switch_plan, resolution) {
        run(command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{Location, Output};
    use super::super::utils::assert_command_eq;

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
                name: "HDMI-A-2".to_string(),
                connected: true,
                enabled: false,
                modes: Vec::new(),
                location: Location::External,
            },
        ];

        let switch_plan = SwitchPlan {
            outputs_to_disable: Vec::new(),
            outputs_to_enable: vec![&outputs[0], &outputs[1]],
        };

        let resolution = None;

        // Act
        let commands = build_switch_commands(&switch_plan, resolution);

        // Assert
        assert!(commands.len() == 2);
        assert_command_eq(&commands[0], "swaymsg", &["output \"eDP-1\" enable position 0 0"]);
        assert_command_eq(&commands[1], "swaymsg", &["output \"HDMI-A-2\" enable position 0 0"]);
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
                name: "HDMI-A-2".to_string(),
                connected: true,
                enabled: true,
                modes: Vec::new(),
                location: Location::External,
            },
        ];

        let switch_plan = SwitchPlan {
            outputs_to_disable: vec![&outputs[0]],
            outputs_to_enable: vec![&outputs[1]],
        };

        let resolution = Some(Resolution {
            width: 1920,
            height: 1080,
        });

        // Act
        let commands = build_switch_commands(&switch_plan, resolution);

        // Assert
        assert!(commands.len() == 2);
        assert_command_eq(&commands[0], "swaymsg", &["output \"eDP-1\" disable"]);
        assert_command_eq(&commands[1], "swaymsg", &["output \"HDMI-A-2\" enable position 0 0 mode \"1920x1080\""]);
    }
}
