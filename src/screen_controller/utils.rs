use std::process;

pub(super) fn run(mut command: process::Command) -> process::Output {
    log::debug!("Running {command:?}");
    let output = command.output().expect("failed to start");

    log::debug!("Output: {output:?}");

    assert!(
        output.status.success(),
        "xrandr exited with status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
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
