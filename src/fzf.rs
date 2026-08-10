use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

/// Run fzf over `items` and return the chosen one.
///
/// `Ok(None)` means the user cancelled or nothing matched — that is a normal
/// outcome, not a failure. Errors are reserved for fzf being missing or dying.
pub fn select(items: &[String]) -> Result<Option<String>> {
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
            // fzf exits as soon as the user picks, closing the pipe on us.
            // A write failure here just means it stopped listening.
            if writeln!(stdin, "{}", item).is_err() {
                break;
            }
        }
    }

    let output = child.wait_with_output()?;

    // fzf: 0 = selected, 1 = no match, 2 = error, 130 = interrupted
    match output.status.code() {
        Some(0) => {}
        Some(1) | Some(130) => return Ok(None),
        Some(code) => bail!("fzf exited with status {code}"),
        None => bail!("fzf was terminated by a signal"),
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        return Ok(None);
    }

    Ok(Some(selected))
}
