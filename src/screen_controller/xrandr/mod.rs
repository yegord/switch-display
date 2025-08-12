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
        self.command.arg("--output").arg(&output_name);
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
        self.command.arg("--same-as").arg(&output_name);
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

pub(crate) fn get_outputs() -> Screen {
    let status = run(Xrandr::new().command());
    let xrandr_output = String::from_utf8(status.stdout).expect("xrandr output is invalid utf-8");
    parsing::parse(&xrandr_output)
}

fn make_apply_commands(
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
        }).into_iter().flatten();

    disable_commands.chain(enable_commands).collect()
}

pub(crate) fn apply(switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
    for command in make_apply_commands(switch_plan, resolution) {
        run(command);
    }
}

fn run(mut command: process::Command) -> process::Output {
    let output = command.output().expect("failed to start");

    assert!(
        output.status.success(),
        "xrandr exited with status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    output
}

#[cfg(test)]
mod tests {
}