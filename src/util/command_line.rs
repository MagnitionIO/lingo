use std::io::{self, Write};
use std::process::{Command, ExitStatus, Stdio};

use crate::util::errors::{BuildResult, LingoError};
use crossbeam::thread;

struct TeeWriter<'a, W0: Write, W1: Write> {
    w0: &'a mut W0,
    w1: &'a mut W1,
}

impl<'a, W0: Write, W1: Write> TeeWriter<'a, W0, W1> {
    fn new(w0: &'a mut W0, w1: &'a mut W1) -> Self {
        Self { w0, w1 }
    }
}

impl<'a, W0: Write, W1: Write> Write for TeeWriter<'a, W0, W1> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // We have to use write_all() otherwise what happens if different
        // amounts are written?
        self.w0.write_all(buf)?;
        self.w1.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w0.flush()?;
        self.w1.flush()?;
        Ok(())
    }
}

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
