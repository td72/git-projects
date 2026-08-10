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
        let targets = parse_targets(env::var("GIT_PROJECTS_TARGETS").ok().as_deref());
        Ok(Self { root, targets })
    }
}

fn resolve_root() -> Result<String> {
    if let Some(root) = ghq_root() {
        return Ok(expand_tilde(&root, env::var("HOME").ok().as_deref()));
    }

    let home = env::var("HOME").context("HOME environment variable not set")?;
    Ok(format!("{}/src", home))
}

fn ghq_root() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", "ghq.root"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(root)
    }
}

fn expand_tilde(path: &str, home: Option<&str>) -> String {
    let Some(home) = home else {
        return path.to_string();
    };

    if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{}", home, rest)
    } else if path == "~" {
        home.to_string()
    } else {
        path.to_string()
    }
}

/// Split a `GIT_PROJECTS_TARGETS` value into filter fragments.
///
/// Empty fragments are dropped: `""` is a substring of every path, so keeping
/// one would silently disable the filter.
fn parse_targets(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_leading_tilde() {
        assert_eq!(expand_tilde("~/src", Some("/home/me")), "/home/me/src");
        assert_eq!(expand_tilde("~", Some("/home/me")), "/home/me");
    }

    #[test]
    fn leaves_other_paths_alone() {
        assert_eq!(expand_tilde("/opt/src", Some("/home/me")), "/opt/src");
        // A tilde that is not a home reference stays literal.
        assert_eq!(expand_tilde("~user/src", Some("/home/me")), "~user/src");
        assert_eq!(expand_tilde("src/~", Some("/home/me")), "src/~");
    }

    #[test]
    fn keeps_tilde_when_home_is_unknown() {
        assert_eq!(expand_tilde("~/src", None), "~/src");
    }

    #[test]
    fn parses_colon_separated_targets() {
        assert_eq!(
            parse_targets(Some("github.com/td72:github.com/myorg")),
            vec!["github.com/td72", "github.com/myorg"]
        );
    }

    #[test]
    fn treats_missing_or_empty_value_as_no_filter() {
        assert!(parse_targets(None).is_empty());
        assert!(parse_targets(Some("")).is_empty());
        assert!(parse_targets(Some(":")).is_empty());
    }

    #[test]
    fn drops_empty_fragments() {
        assert_eq!(
            parse_targets(Some(":github.com/td72::")),
            vec!["github.com/td72"]
        );
    }
}
