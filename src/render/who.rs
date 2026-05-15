use crate::analyzer::ContributorStats;
use std::io::{self, Write};

use super::common::{format_ratio_bar, RatioBarColor, WHO_BAR_WIDTH};
use super::style::{bold, ColorOutput};

/// Input for rendering the `who` command.
pub struct WhoReport<'a> {
    pub repo_name: &'a str,
    pub contributors: &'a [ContributorStats],
}

fn format_who_header(repo_name: &str) -> String {
    format!("  {}  ·  who  ·  contributors\n\n", repo_name)
}

fn format_who_row(author_width: usize, contributor: &ContributorStats, max_commits: u32) -> String {
    let bar = format_ratio_bar(
        contributor.commits,
        max_commits,
        WHO_BAR_WIDTH,
        RatioBarColor::Green,
        ColorOutput::AutoTerminal,
    );
    format!(
        "  {:author_width$}  {}  {} commits  feat {}%  fix {}%  chore {}%  other {}%\n",
        contributor.author,
        bar,
        bold(&contributor.commits.to_string()),
        contributor.feat_pct,
        contributor.fix_pct,
        contributor.chore_pct,
        contributor.other_pct,
    )
}

/// Renders the full `who` report to a string.
pub fn render_who(report: &WhoReport) -> String {
    let mut out = format_who_header(report.repo_name);
    if report.contributors.is_empty() {
        return out;
    }

    let author_width = report
        .contributors
        .iter()
        .map(|c| c.author.chars().count())
        .max()
        .unwrap_or(0);
    let max_commits = report
        .contributors
        .iter()
        .map(|c| c.commits)
        .max()
        .unwrap_or(0);

    for contributor in report.contributors {
        out.push_str(&format_who_row(author_width, contributor, max_commits));
        out.push('\n');
    }

    out
}

/// Renders the `who` report and writes it to stdout.
pub fn print_who(report: &WhoReport) {
    let _ = io::stdout().write_all(render_who(report).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::common::{format_ratio_bar, RatioBarColor};
    use crate::render::style::ColorOutput;

    fn strip_ansi(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                while chars.next().is_some_and(|c| c != 'm') {}
                continue;
            }
            result.push(c);
        }
        result
    }

    #[test]
    fn format_ratio_bar_small_value_shows_one_block() {
        let bar = format_ratio_bar(1, 10_000, 12, RatioBarColor::Plain, ColorOutput::Never);
        assert_eq!(bar.matches('█').count(), 1);
        assert_eq!(bar.chars().count(), 12);
    }

    #[test]
    fn format_ratio_bar_scales_to_width() {
        let bar = format_ratio_bar(8, 10, 10, RatioBarColor::Plain, ColorOutput::Never);
        assert_eq!(bar.chars().count(), 10);
        assert_eq!(bar.matches('█').count(), 8);
        assert_eq!(bar.matches('░').count(), 2);
    }

    #[test]
    fn render_who_contains_expected_sections() {
        let contributors = vec![
            ContributorStats {
                author: "alice@corp.com".into(),
                commits: 312,
                feat_pct: 61,
                fix_pct: 28,
                chore_pct: 11,
                other_pct: 0,
            },
            ContributorStats {
                author: "bob@corp.com".into(),
                commits: 198,
                feat_pct: 44,
                fix_pct: 41,
                chore_pct: 15,
                other_pct: 0,
            },
        ];
        let plain = strip_ansi(&render_who(&WhoReport {
            repo_name: "my-repo",
            contributors: &contributors,
        }));

        assert!(plain.contains("my-repo"));
        assert!(plain.contains("who"));
        assert!(plain.contains("contributors"));
        assert!(plain.contains("alice@corp.com"));
        assert!(plain.contains("312 commits"));
        assert!(plain.contains("feat 61%"));
        assert!(plain.contains("other 0%"));
    }
}
