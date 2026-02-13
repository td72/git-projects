use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn history_path() -> PathBuf {
    let dir = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".local/share")
        });
    dir.join("git-projects/history")
}

pub fn load() -> Vec<String> {
    let path = history_path();
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn record(selected: &str) {
    let path = history_path();
    let prev = load();

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let Ok(mut file) = fs::File::create(&path) else {
        return;
    };

    // Selected item goes to the top
    let _ = writeln!(file, "{}", selected);
    for entry in &prev {
        if entry != selected {
            let _ = writeln!(file, "{}", entry);
        }
    }
}

/// Sort repos: history order first, then the rest alphabetically.
pub fn sort_by_history(repos: Vec<String>) -> Vec<String> {
    let history = load();
    let repo_set: HashSet<&str> = repos.iter().map(|s| s.as_str()).collect();

    let mut sorted = Vec::with_capacity(repos.len());

    // History items first (only if they still exist)
    let mut seen = HashSet::new();
    for h in &history {
        if repo_set.contains(h.as_str()) && seen.insert(h.as_str()) {
            sorted.push(h.clone());
        }
    }

    // Remaining repos in alphabetical order
    for r in &repos {
        if !seen.contains(r.as_str()) {
            sorted.push(r.clone());
        }
    }

    sorted
}
