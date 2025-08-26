use std::process;

pub(super) fn run(mut command: process::Command) -> process::Output {
    log::debug!("Running {command:?}");
    let output = command.output().expect("failed to start");

    log::debug!("Output: {output:?}");

    assert!(
        output.status.success(),
        "{command:?} exited with {output:?}"
    );

    output
}

#[cfg(test)]
pub(super) fn assert_command_eq(
    actual: &std::process::Command,
    expected_program: &str,
    expected_args: &[&str],
) {
    assert_eq!(
        actual
            .get_program()
            .to_str()
            .expect("program name is not valid utf-8"),
        expected_program
    );

    let actual_args: Vec<&str> = actual
        .get_args()
        .map(|arg| arg.to_str().expect("argument is not valid utf-8"))
        .collect();

    assert_eq!(actual_args, expected_args);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_smoke_test() {
        // Arrange
        let mut command = process::Command::new("echo");
        command.arg("OK");

        // Act
        let output = run(command);

        // Assert
        assert_eq!(output.stdout, b"OK\n");
    }
}
