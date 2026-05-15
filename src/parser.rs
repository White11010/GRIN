#[derive(Debug)]
pub struct Commit {
    /// Full commit hash from `git log` (kept for future features / debugging).
    #[allow(dead_code)]
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

impl Commit {
    pub fn new(hash: String, author: String, date: String, message: String) -> Self {
        Self {
            hash,
            author,
            date,
            message,
        }
    }
}

pub fn parse_log(log: &str) -> Result<Vec<Commit>, std::io::Error> {
    let mut commits = Vec::new();
    for line in log.lines() {
        let parts = line.split('|').collect::<Vec<&str>>();
        if parts.len() != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid log format",
            ));
        }
        commits.push(Commit::new(
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
            parts[3].to_string(),
        ));
    }
    Ok(commits)
}

fn normalize_path(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let path = if let Some((_, new)) = line.split_once("=>") {
        new.trim()
    } else {
        line
    };
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

/// One inner vec = unique file paths touched in a single commit.
pub fn parse_file_touches(log: &str) -> Result<Vec<Vec<String>>, std::io::Error> {
    use std::collections::HashSet;

    let mut commits = Vec::new();
    let mut current: HashSet<String> = HashSet::new();

    let flush = |current: &mut HashSet<String>, commits: &mut Vec<Vec<String>>| {
        if !current.is_empty() {
            commits.push(current.drain().collect());
        }
    };

    for line in log.lines() {
        if line.trim().is_empty() {
            flush(&mut current, &mut commits);
        } else if let Some(path) = normalize_path(line) {
            current.insert(path);
        }
    }
    flush(&mut current, &mut commits);

    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_touches_splits_commits_and_dedups() {
        let log = "src/a.rs\nsrc/b.rs\n\nsrc/a.rs\nsrc/a.rs\n\n";
        let touches = parse_file_touches(log).unwrap();
        assert_eq!(touches.len(), 2);
        assert_eq!(touches[0].len(), 2);
        assert_eq!(touches[1].len(), 1);
        assert_eq!(touches[1][0], "src/a.rs");
    }

    #[test]
    fn parse_file_touches_handles_rename() {
        let log = "old/path.rs => new/path.rs\n\n";
        let touches = parse_file_touches(log).unwrap();
        assert_eq!(touches[0], vec!["new/path.rs".to_string()]);
    }
}
