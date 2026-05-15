use std::collections::HashMap;

/// File change frequency for the `churn` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChurn {
    pub path: String,
    pub changes: u32,
}

/// Summary statistics for the `churn` command footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChurnSummary {
    pub commits_scanned: usize,
    pub unique_files: usize,
}

fn path_extension(path: &str) -> Option<&str> {
    let file_name = path.rsplit(['/', '\\']).next()?;
    let (_, ext) = file_name.rsplit_once('.')?;
    if ext.is_empty() {
        None
    } else {
        Some(ext)
    }
}

fn path_matches_extensions(path: &str, extensions: &[String]) -> bool {
    let Some(ext) = path_extension(path) else {
        return false;
    };
    extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
}

/// Returns top files by commit touch count and overall summary.
///
/// When `extensions` is `Some`, only paths whose file extension (last segment after `.`) is in
/// the list (ASCII case-insensitive) are counted.
pub fn file_churn_stats(
    touches_per_commit: &[Vec<String>],
    limit: usize,
    extensions: Option<&[String]>,
) -> (Vec<FileChurn>, ChurnSummary) {
    let mut changes: HashMap<String, u32> = HashMap::new();

    for files in touches_per_commit {
        for path in files {
            if let Some(exts) = extensions {
                if !path_matches_extensions(path, exts) {
                    continue;
                }
            }
            *changes.entry(path.clone()).or_insert(0) += 1;
        }
    }

    let summary = ChurnSummary {
        commits_scanned: touches_per_commit.len(),
        unique_files: changes.len(),
    };

    let mut files: Vec<FileChurn> = changes
        .into_iter()
        .map(|(path, changes)| FileChurn { path, changes })
        .collect();

    files.sort_by(|a, b| {
        b.changes
            .cmp(&a.changes)
            .then_with(|| a.path.cmp(&b.path))
    });
    files.truncate(limit);

    (files, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_churn_stats_counts_and_limits() {
        let touches = vec![
            vec!["a.rs".into(), "b.rs".into()],
            vec!["a.rs".into()],
            vec!["c.rs".into()],
        ];

        let (files, summary) = file_churn_stats(&touches, 2, None);
        assert_eq!(summary.commits_scanned, 3);
        assert_eq!(summary.unique_files, 3);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "a.rs");
        assert_eq!(files[0].changes, 2);
    }

    #[test]
    fn file_churn_stats_filters_by_extension() {
        let touches = vec![
            vec!["a.rs".into(), "b.ts".into()],
            vec!["a.rs".into(), "c.tsx".into()],
        ];
        let exts = vec!["ts".into(), "tsx".into()];
        let (files, summary) = file_churn_stats(&touches, 10, Some(&exts));
        assert_eq!(summary.commits_scanned, 2);
        assert_eq!(summary.unique_files, 2);
        let paths: Vec<_> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"b.ts"));
        assert!(paths.contains(&"c.tsx"));
        assert!(!paths.contains(&"a.rs"));
    }

    #[test]
    fn file_churn_stats_extension_matches_case_insensitive() {
        let touches = vec![vec!["x.YAML".into()]];
        let exts = vec!["yaml".into()];
        let (files, _) = file_churn_stats(&touches, 5, Some(&exts));
        assert_eq!(files.len(), 1);
    }
}
