use crate::analyzer::{ChurnSummary, FileChurn};
use std::io::{self, Write};

use super::common::{CHURN_BAR_WIDTH, RatioBarColor, format_ratio_bar};
use super::style::{ColorOutput, bold};

/// Input for rendering the `churn` command.
pub struct ChurnReport<'a> {
    pub repo_name: &'a str,
    pub files: &'a [FileChurn],
    pub summary: &'a ChurnSummary,
    pub extensions_filter: Option<&'a str>,
}

fn format_churn_header(repo_name: &str, extensions_filter: Option<&str>) -> String {
    let mut header = format!(
        "  {}  ·  churn  ·  top files by change frequency\n",
        repo_name
    );
    if let Some(exts) = extensions_filter {
        header.push_str(&format!("    extensions — {}\n", exts));
    }
    header.push('\n');
    header
}

fn format_churn_row(path_width: usize, file: &FileChurn, max_changes: u32) -> String {
    let bar = format_ratio_bar(
        file.changes,
        max_changes,
        CHURN_BAR_WIDTH,
        RatioBarColor::Red,
        ColorOutput::AutoTerminal,
    );
    format!(
        "  {:path_width$}  {}  {} changes\n",
        file.path,
        bar,
        bold(&file.changes.to_string()),
    )
}

fn format_churn_footer(summary: &ChurnSummary) -> String {
    format!(
        "\n  {} commits scanned  ·  {} unique files\n",
        bold(&summary.commits_scanned.to_string()),
        bold(&summary.unique_files.to_string()),
    )
}

/// Renders the full `churn` report to a string.
pub fn render_churn(report: &ChurnReport) -> String {
    let mut out = format_churn_header(report.repo_name, report.extensions_filter);
    if report.files.is_empty() {
        out.push_str(&format_churn_footer(report.summary));
        return out;
    }

    let path_width = report
        .files
        .iter()
        .map(|f| f.path.chars().count())
        .max()
        .unwrap_or(0);
    let max_changes = report.files.iter().map(|f| f.changes).max().unwrap_or(0);

    for file in report.files {
        out.push_str(&format_churn_row(path_width, file, max_changes));
        out.push('\n');
    }

    out.push_str(&format_churn_footer(report.summary));
    out
}

/// Renders the `churn` report and writes it to stdout.
pub fn print_churn(report: &ChurnReport) {
    let _ = io::stdout().write_all(render_churn(report).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn render_churn_contains_expected_sections() {
        let files = vec![FileChurn {
            path: "src/auth/middleware.rs".into(),
            changes: 81,
        }];
        let summary = ChurnSummary {
            commits_scanned: 847,
            unique_files: 312,
        };
        let plain = strip_ansi(&render_churn(&ChurnReport {
            repo_name: "my-repo",
            files: &files,
            summary: &summary,
            extensions_filter: None,
        }));

        assert!(plain.contains("churn"));
        assert!(plain.contains("top files by change frequency"));
        assert!(plain.contains("src/auth/middleware.rs"));
        assert!(plain.contains("81 changes"));
        assert!(plain.contains("847 commits scanned"));
        assert!(plain.contains("312 unique files"));
    }
}
