use anyhow::{Context, Result};
use std::env;
use std::process::Command;

pub struct AppContext {
    pub root: String,
    pub targets: Vec<String>,
}

impl AppContext {
    pub fn load() -> Result<Self> {
        let root = resolve_root()?;
        let targets = parse_targets();
        Ok(Self { root, targets })
    }
}

fn resolve_root() -> Result<String> {
    let output = Command::new("git")
        .args(["config", "--global", "ghq.root"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !root.is_empty() {
                return Ok(expand_tilde(&root));
            }
        }
    }

    let home = env::var("HOME").context("HOME environment variable not set")?;
    Ok(format!("{}/src", home))
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{}/{}", home, rest);
        }
    } else if path == "~" {
        if let Ok(home) = env::var("HOME") {
            return home;
        }
    }
    path.to_string()
}

fn parse_targets() -> Vec<String> {
    match env::var("GIT_PROJECTS_TARGETS") {
        Ok(val) if !val.is_empty() => val.split(':').map(|s| s.to_string()).collect(),
        _ => vec![],
    }
}
