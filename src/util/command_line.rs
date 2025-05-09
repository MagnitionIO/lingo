use std::io::{self};
use std::process::{Command, ExitStatus, Stdio};

use crate::util::errors::{BuildResult, LingoError};

pub fn run_and_capture(command: &mut Command) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    // command.stdout(Stdio::piped());
    // command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let status = child.wait().expect("child wasn't running");

    let stdout_log = Vec::new();
    let stderr_log = Vec::new();

    Ok((status, stdout_log, stderr_log))
}

pub fn execute_command_to_build_result(mut command: Command) -> BuildResult {
    match run_and_capture(&mut command) {
        Err(e) => {
            log::error!("error occured while executing commandline: {:?}", &e);
            Err(Box::new(e))
        }
        Ok((status, _, _)) if !status.success() => {
            Err(Box::new(LingoError::CommandFailed(command, status)))
        }
        _ => Ok(()),
    }
}
