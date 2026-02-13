use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn select(items: &[String]) -> Result<String> {
    let mut child = Command::new("fzf")
        .args(["--reverse", "--height", "20", "--prompt", "cd > "])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start fzf. Is it installed?")?;

    {
        let stdin = child.stdin.as_mut().context("failed to open fzf stdin")?;
        for item in items {
            writeln!(stdin, "{}", item)?;
        }
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        bail!("fzf was cancelled");
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        bail!("no item selected");
    }

    Ok(selected)
}
