use std::path::Path;
use walkdir::WalkDir;

pub fn list_repos(root: &str) -> Vec<String> {
    let root_path = Path::new(root);
    let mut repos = Vec::new();

    let mut walker = WalkDir::new(root).into_iter();
    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        // Skip hidden directories
        if entry.depth() > 0
            && entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        {
            walker.skip_current_dir();
            continue;
        }

        // Check if this directory is a git repo
        if entry.path().join(".git").is_dir() {
            if let Ok(rel) = entry.path().strip_prefix(root_path) {
                let rel_str = rel.to_string_lossy().to_string();
                if !rel_str.is_empty() {
                    repos.push(rel_str);
                }
            }
            // Don't descend into the repo's subdirectories
            walker.skip_current_dir();
        }
    }

    repos.sort();
    repos
}
