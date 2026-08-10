use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// How many entries to keep. Beyond this the tail is old enough that it no
/// longer affects the ordering anyone notices.
const MAX_ENTRIES: usize = 100;

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
    load_from(&history_path())
}

fn load_from(path: &Path) -> Vec<String> {
    let file = match fs::File::open(path) {
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
    record_to(&history_path(), selected);
}

fn record_to(path: &Path, selected: &str) {
    if selected.is_empty() {
        return;
    }

    let prev = load_from(path);

    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }

    // Write to a sibling and rename, so an interrupted run cannot leave a
    // truncated history behind.
    let tmp = path.with_extension("tmp");
    let Ok(file) = fs::File::create(&tmp) else {
        return;
    };

    let write = || -> std::io::Result<()> {
        let mut out = BufWriter::new(file);
        // Selected item goes to the top
        writeln!(out, "{}", selected)?;

        let mut seen = HashSet::from([selected]);
        for entry in prev.iter().take(MAX_ENTRIES * 4) {
            if seen.len() >= MAX_ENTRIES {
                break;
            }
            if seen.insert(entry.as_str()) {
                writeln!(out, "{}", entry)?;
            }
        }
        out.flush()
    };

    if write().is_err() || fs::rename(&tmp, path).is_err() {
        let _ = fs::remove_file(&tmp);
    }
}

/// Sort repos: history order first, then the rest alphabetically.
pub fn sort_by_history(repos: Vec<String>) -> Vec<String> {
    sort_with_history(repos, &load())
}

fn sort_with_history(repos: Vec<String>, history: &[String]) -> Vec<String> {
    let repo_set: HashSet<&str> = repos.iter().map(|s| s.as_str()).collect();

    let mut sorted = Vec::with_capacity(repos.len());

    // History items first (only if they still exist)
    let mut seen = HashSet::new();
    for h in history {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn puts_history_first_then_the_rest() {
        let repos = strings(&["a", "b", "c", "d"]);
        let history = strings(&["c", "a"]);

        assert_eq!(
            sort_with_history(repos, &history),
            strings(&["c", "a", "b", "d"])
        );
    }

    #[test]
    fn ignores_history_entries_that_no_longer_exist() {
        let repos = strings(&["a", "b"]);
        let history = strings(&["gone", "b"]);

        assert_eq!(sort_with_history(repos, &history), strings(&["b", "a"]));
    }

    #[test]
    fn tolerates_duplicate_history_entries() {
        let repos = strings(&["a", "b"]);
        let history = strings(&["b", "b", "a"]);

        assert_eq!(sort_with_history(repos, &history), strings(&["b", "a"]));
    }

    #[test]
    fn empty_history_keeps_the_input_order() {
        let repos = strings(&["a", "b"]);

        assert_eq!(sort_with_history(repos, &[]), strings(&["a", "b"]));
    }

    #[test]
    fn records_the_selection_at_the_top_without_duplicating_it() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nested/history");

        record_to(&path, "a");
        record_to(&path, "b");
        record_to(&path, "a");

        assert_eq!(load_from(&path), strings(&["a", "b"]));
    }

    #[test]
    fn caps_the_file_length() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("history");

        for i in 0..(MAX_ENTRIES + 50) {
            record_to(&path, &format!("repo-{}", i));
        }

        let entries = load_from(&path);
        assert_eq!(entries.len(), MAX_ENTRIES);
        // Most recent first.
        assert_eq!(entries[0], format!("repo-{}", MAX_ENTRIES + 49));
    }

    #[test]
    fn missing_file_loads_as_empty() {
        let tmp = tempfile::tempdir().unwrap();

        assert!(load_from(&tmp.path().join("absent")).is_empty());
    }

    #[test]
    fn leaves_no_temp_file_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("history");

        record_to(&path, "a");

        assert!(!path.with_extension("tmp").exists());
    }
}
