use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

/// Executes the clipboard command
pub fn copy_to_clipboard(text: &str, cmd: &str) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to run command with exit code {}.", status).into())
    }
}
