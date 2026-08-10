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

        // Check if this directory is a git repo. `.git` is a directory in a
        // normal clone but a file in a worktree or submodule, so test for
        // existence rather than for a directory.
        if entry.path().join(".git").exists() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo_dir(root: &Path, rel: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.join(".git")).unwrap();
    }

    fn worktree(root: &Path, rel: &str, gitdir: &str) {
        let path = root.join(rel);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join(".git"), format!("gitdir: {}\n", gitdir)).unwrap();
    }

    #[test]
    fn finds_repos_by_relative_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        repo_dir(root, "github.com/td72/git-projects");
        repo_dir(root, "github.com/myorg/thing");

        assert_eq!(
            list_repos(root.to_str().unwrap()),
            vec!["github.com/myorg/thing", "github.com/td72/git-projects",]
        );
    }

    #[test]
    fn finds_worktrees_where_dot_git_is_a_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        worktree(root, "github.com/td72/wt", "/elsewhere/.git/worktrees/wt");

        assert_eq!(
            list_repos(root.to_str().unwrap()),
            vec!["github.com/td72/wt"]
        );
    }

    #[test]
    fn does_not_descend_into_a_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        repo_dir(root, "outer");
        repo_dir(root, "outer/vendor/inner");

        assert_eq!(list_repos(root.to_str().unwrap()), vec!["outer"]);
    }

    #[test]
    fn skips_hidden_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        repo_dir(root, ".cache/hidden-repo");
        repo_dir(root, "visible");

        assert_eq!(list_repos(root.to_str().unwrap()), vec!["visible"]);
    }

    #[test]
    fn ignores_plain_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("not-a-repo/nested")).unwrap();

        assert!(list_repos(root.to_str().unwrap()).is_empty());
    }

    #[test]
    fn missing_root_yields_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope");

        assert!(list_repos(missing.to_str().unwrap()).is_empty());
    }
}
