use crate::parser::Commit;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitKind {
    Feat,
    Fix,
    Chore,
    Other,
}

/// Aggregated contributor metrics for the `who` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributorStats {
    pub author: String,
    pub commits: u32,
    pub feat_pct: u32,
    pub fix_pct: u32,
    pub chore_pct: u32,
    pub other_pct: u32,
}

fn matches_commit_type(message: &str, ty: &str) -> bool {
    let m = message.trim();
    if m.eq_ignore_ascii_case(ty) {
        return true;
    }
    let prefix_lower: String = m.chars().take(ty.len()).collect::<String>().to_lowercase();
    if prefix_lower != ty {
        return false;
    }
    m.chars()
        .nth(ty.len())
        .is_some_and(|c| c == ':' || c == '(')
}

fn commit_kind(message: &str) -> CommitKind {
    if matches_commit_type(message, "feat") {
        CommitKind::Feat
    } else if matches_commit_type(message, "fix") {
        CommitKind::Fix
    } else if matches_commit_type(message, "chore") {
        CommitKind::Chore
    } else {
        CommitKind::Other
    }
}

fn pct(count: u32, total: u32) -> u32 {
    if total == 0 {
        0
    } else {
        ((count as f64 / total as f64) * 100.0).round() as u32
    }
}

/// Returns top contributors by commit count.
pub fn contributor_stats(commits: &[Commit], limit: usize) -> Vec<ContributorStats> {
    let mut by_author: HashMap<String, (u32, u32, u32, u32, u32)> = HashMap::new();

    for commit in commits {
        let entry = by_author
            .entry(commit.author.clone())
            .or_insert((0, 0, 0, 0, 0));
        entry.0 += 1;
        match commit_kind(&commit.message) {
            CommitKind::Feat => entry.1 += 1,
            CommitKind::Fix => entry.2 += 1,
            CommitKind::Chore => entry.3 += 1,
            CommitKind::Other => entry.4 += 1,
        }
    }

    let mut stats: Vec<ContributorStats> = by_author
        .into_iter()
        .map(
            |(author, (total, feat, fix, chore, other))| ContributorStats {
                author,
                commits: total,
                feat_pct: pct(feat, total),
                fix_pct: pct(fix, total),
                chore_pct: pct(chore, total),
                other_pct: pct(other, total),
            },
        )
        .collect();

    stats.sort_by(|a, b| {
        b.commits
            .cmp(&a.commits)
            .then_with(|| a.author.cmp(&b.author))
    });
    stats.truncate(limit);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(date: &str, author: &str, message: &str) -> Commit {
        Commit::new("hash".into(), author.into(), date.into(), message.into())
    }

    #[test]
    fn contributor_stats_sorts_and_limits() {
        let commits = vec![
            commit("2020-01-01", "alice@corp.com", "feat: one"),
            commit("2020-02-01", "alice@corp.com", "fix: two"),
            commit("2020-03-01", "bob@corp.com", "chore: three"),
        ];

        let stats = contributor_stats(&commits, 1);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].author, "alice@corp.com");
        assert_eq!(stats[0].commits, 2);
    }

    #[test]
    fn contributor_stats_classifies_commit_types() {
        let commits = vec![
            commit("2020-01-01", "alice@corp.com", "feat: add"),
            commit("2020-02-01", "alice@corp.com", "fix: bug"),
            commit("2020-03-01", "alice@corp.com", "docs: readme"),
        ];

        let stats = contributor_stats(&commits, 5);
        assert_eq!(stats[0].feat_pct, 33);
        assert_eq!(stats[0].fix_pct, 33);
        assert_eq!(stats[0].chore_pct, 0);
        assert_eq!(stats[0].other_pct, 33);
    }
}
