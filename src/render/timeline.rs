use crate::analyzer::Event;
use crate::parser::Commit;
use std::collections::HashSet;
use std::io::{self, Write};

use super::style::{blue, bold, cyan, dim_red, use_color, yellow, green};

const DATE_WIDTH: usize = 8;
const SPINE_INDENT: &str = "           ";
const RULE_WIDTH: usize = 54;
const LINE_WIDTH: usize = 54;
const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Input for rendering a sorted project timeline.
pub struct Timeline<'a> {
    pub repo_name: &'a str,
    pub commits: &'a [Commit],
    pub events: &'a [Event],
}

/// Aggregated values for the timeline header and footer.
struct TimelineStats {
    duration: String,
    contributors: usize,
    commit_count: usize,
    peak_label: Option<String>,
    longest_silence: Option<u32>,
}

/// Parses a commit date into `(year, month, day)`.
fn parse_date(date: &str) -> (u32, u32, u32) {
    let date = date.split_whitespace().next().unwrap_or(date);
    let mut parts = date.split('-');
    let year = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let month = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let day = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (year, month, day)
}

/// Returns `YYYY-MM` from a full commit date string.
fn parse_date_prefix(date: &str) -> String {
    let date = date.split_whitespace().next().unwrap_or(date);
    date.chars().take(7).collect()
}

/// Formats a short month name and year (e.g. `Feb 2020`).
fn month_name_short(year: u32, month: u32) -> String {
    let name = MONTH_NAMES
        .get((month as usize).saturating_sub(1))
        .copied()
        .unwrap_or("???");
    format!("{name} {year}")
}

/// Formats calendar duration between two dates as `N years M months`.
fn duration_between(start: &str, end: &str) -> String {
    let (sy, sm, _) = parse_date(start);
    let (ey, em, _) = parse_date(end);
    let total_months = (ey as i32 * 12 + em as i32) - (sy as i32 * 12 + sm as i32);
    let years = (total_months / 12).max(0) as u32;
    let months = (total_months % 12).max(0) as u32;

    match (years, months) {
        (0, 0) => "0 months".to_string(),
        (0, 1) => "1 month".to_string(),
        (0, m) => format!("{m} months"),
        (1, 0) => "1 year".to_string(),
        (1, 1) => "1 year 1 month".to_string(),
        (1, m) => format!("1 year {m} months"),
        (y, 0) => format!("{y} years"),
        (y, 1) => format!("{y} years 1 month"),
        (y, m) => format!("{y} years {m} months"),
    }
}

/// Builds header/footer statistics from commits and events.
fn timeline_stats(commits: &[Commit], events: &[Event]) -> TimelineStats {
    let contributors: HashSet<_> = commits.iter().map(|c| &c.author).collect();
    let commit_count = commits.len();

    let duration = match (commits.first(), commits.last()) {
        (Some(first), Some(last)) => duration_between(&first.date, &last.date),
        _ => "0 months".to_string(),
    };

    let peak_label = events.iter().find_map(|e| match e {
        Event::Peak { date, .. } => {
            let (year, month, _) = parse_date(&format!("{date}-01"));
            Some(month_name_short(year, month))
        }
        _ => None,
    });

    let longest_silence = events
        .iter()
        .filter_map(|e| match e {
            Event::Silence { months, .. } => Some(*months),
            _ => None,
        })
        .max();

    TimelineStats {
        duration,
        contributors: contributors.len(),
        commit_count,
        peak_label,
        longest_silence,
    }
}

/// Renders a 12-month sparkline from commit counts.
fn format_sparkline(chart: [u32; 12]) -> String {
    let max = chart.iter().copied().max().unwrap_or(0).max(1);
    chart
        .iter()
        .map(|&count| {
            let level = ((count as f64 / max as f64) * 8.0).round() as usize;
            BLOCKS[level.min(8)]
        })
        .collect()
}

/// Pads a label to the date column width.
fn pad_date_label(label: &str) -> String {
    format!("{label:DATE_WIDTH$}")
}

/// Renders the horizontal rule used in header and footer.
fn format_rule() -> String {
    format!("  {}\n", "─".repeat(RULE_WIDTH))
}

/// Renders the timeline header line.
fn format_header(repo_name: &str, stats: &TimelineStats) -> String {
    format!(
        "  {}  ·  timeline  ·  {}\n\n",
        repo_name, stats.duration
    )
}

/// Renders the timeline footer with summary statistics.
fn format_footer(stats: &TimelineStats) -> String {
    let mut parts = vec![
        format!("{} contributors", bold(&stats.contributors.to_string())),
        format!("{} commits", bold(&stats.commit_count.to_string())),
    ];

    if let Some(ref peak) = stats.peak_label {
        parts.push(format!("peak: {}", bold(peak)));
    }

    if let Some(months) = stats.longest_silence {
        parts.push(format!(
            "longest silence: {}",
            bold(&format!("{months} mo"))
        ));
    }

    format!("  {}\n", parts.join("  ·  "))
}

/// Formats the silence suffix with dashed fill to the target line width.
fn format_silence_suffix(months: u32, prefix_len: usize) -> String {
    let label = if months == 1 {
        "1 month silence".to_string()
    } else {
        format!("{months} month silence")
    };
    let head = format!("  ╌╌ {label} ");
    let fill_len = LINE_WIDTH.saturating_sub(prefix_len + head.chars().count());
    format!("{head}{}", "╌".repeat(fill_len))
}

/// Renders a single timeline event line (without spine connector).
fn format_event_line(event: &Event, commits: &[Commit]) -> String {
    match event {
        Event::Born {
            date,
            message,
            author,
        } => {
            let label = pad_date_label(&parse_date_prefix(date));
            format!(
                "  {}  {}  \"{}\"  {}",
                label,
                cyan("◆ born"),
                message,
                author
            )
        }
        Event::Joined { date, author } => {
            let label = pad_date_label(&parse_date_prefix(date));
            format!(
                "  {}  {} {} joined",
                label,
                green("+"),
                green(author)
            )
        }
        Event::Peak { date, count } => {
            let label = pad_date_label(date);
            format!(
                "  {}  {}  — {} commits this month",
                label,
                yellow("◆ peak"),
                bold(&count.to_string())
            )
        }
        Event::Silence { from, months } => {
            let label = pad_date_label(&parse_date_prefix(from));
            let prefix_len = label.len() + 2;
            let suffix = format_silence_suffix(*months, prefix_len);
            format!("  {label}{}", dim_red(&suffix))
        }
        Event::Revival { date, msg } => {
            let label = pad_date_label(&parse_date_prefix(date));
            let author = commits
                .iter()
                .find(|c| c.message == *msg)
                .map(|c| c.author.as_str())
                .unwrap_or("");
            format!(
                "  {}  {}  \"{}\"  {}",
                label,
                cyan("◆ revival"),
                msg,
                author
            )
        }
        Event::Latest {
            date,
            message,
            author,
        } => {
            let label = pad_date_label(&parse_date_prefix(date));
            format!(
                "  {}  {}  \"{}\"  {}",
                label,
                cyan("◆ latest"),
                message,
                author
            )
        }
        Event::EndOfYear { year, chart } => {
            let label = pad_date_label(&year.to_string());
            let sparkline = format_sparkline(*chart);
            format!(
                "  {} │  {}",
                label,
                if use_color() {
                    blue(&sparkline)
                } else {
                    sparkline
                }
            )
        }
    }
}

/// Renders a spine connector line between events.
fn format_spine(is_last: bool) -> String {
    let ch = if is_last { '·' } else { '│' };
    format!("{SPINE_INDENT}{ch}\n")
}

/// Renders the full timeline to a string.
pub fn render_timeline(timeline: &Timeline) -> String {
    let stats = timeline_stats(timeline.commits, timeline.events);
    let mut out = String::new();

    out.push_str(&format_header(timeline.repo_name, &stats));
    out.push_str(&format_rule());

    let last_index = timeline.events.len().saturating_sub(1);
    for (i, event) in timeline.events.iter().enumerate() {
        out.push_str(&format_event_line(event, timeline.commits));
        out.push('\n');
        out.push_str(&format_spine(i == last_index));
    }

    out.push('\n');
    out.push_str(&format_rule());
    out.push_str(&format_footer(&stats));

    out
}

/// Renders the timeline and writes it to stdout.
pub fn print_timeline(timeline: &Timeline) {
    let output = render_timeline(timeline);
    let _ = io::stdout().write_all(output.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer;

    fn commit(date: &str, author: &str, message: &str) -> Commit {
        Commit::new(
            "hash".into(),
            author.into(),
            date.into(),
            message.into(),
        )
    }

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
    fn format_sparkline_normalizes_counts() {
        let chart = [0, 1, 2, 0, 47, 0, 0, 0, 0, 0, 0, 0];
        let line = format_sparkline(chart);
        assert_eq!(line.chars().count(), 12);
        assert!(line.contains('█'));
    }

    #[test]
    fn duration_between_years_and_months() {
        assert_eq!(
            duration_between("2019-03-01", "2024-11-15"),
            "5 years 8 months"
        );
    }

    #[test]
    fn render_contains_expected_sections() {
        let commits = vec![
            commit("2019-03-01", "alice@corp.com", "Initial commit"),
            commit("2019-06-15", "carol@corp.com", "Carol joins"),
            commit("2020-02-15", "alice@corp.com", "Peak work"),
            commit("2020-07-15", "alice@corp.com", "Mid year"),
            commit("2020-12-01", "alice@corp.com", "Back to it"),
            commit("2021-03-10", "bob@corp.com", "Bob joins"),
            commit("2024-11-15", "charlie@corp.com", "Fix auth bug"),
        ];
        let events = analyzer::sort_events(analyzer::generate_events_from_commits(&commits));
        let output = render_timeline(&Timeline {
            repo_name: "my-awesome-repo",
            commits: &commits,
            events: &events,
        });
        let plain = strip_ansi(&output);

        assert!(plain.contains("my-awesome-repo"));
        assert!(plain.contains("timeline"));
        assert!(plain.contains("born"));
        assert!(plain.contains("joined"));
        assert!(plain.contains("peak"));
        assert!(plain.contains("contributors"));
        assert!(plain.contains("commits"));

        let born = plain.find("born").expect("born");
        let silence = plain.find("silence").unwrap_or(0);
        let revival = plain.find("revival").unwrap_or(0);
        if silence > 0 && revival > 0 {
            assert!(born < silence);
            assert!(silence < revival);
        }
    }

    #[test]
    fn render_revival_shows_message() {
        let commits = vec![
            commit("2020-07-15", "alice@corp.com", "Last"),
            commit("2020-12-01", "alice@corp.com", "Back to it"),
        ];
        let events = analyzer::sort_events(analyzer::generate_events_from_commits(&commits));
        let plain = strip_ansi(&render_timeline(&Timeline {
            repo_name: "repo",
            commits: &commits,
            events: &events,
        }));
        assert!(plain.contains("revival"));
        assert!(plain.contains("Back to it"));
    }
}
