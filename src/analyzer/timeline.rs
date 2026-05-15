use crate::parser::Commit;
use std::collections::HashMap;

type Year = u32;

#[derive(Debug, Clone)]
pub enum Event {
    Born {
        date: String,
        message: String,
        author: String,
    },
    Joined {
        date: String,
        author: String,
    },
    Peak {
        date: String,
        count: u32,
    },
    Silence {
        from: String,
        months: u32,
    },
    Revival {
        date: String,
        msg: String,
    },
    Latest {
        date: String,
        message: String,
        author: String,
    },
    EndOfYear {
        year: Year,
        chart: [u32; 12],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EventKind {
    EndOfYear = 0,
    Joined = 1,
    Peak = 2,
    Silence = 3,
    Revival = 4,
}

/// Calendar year and month (1–12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct YearMonth {
    year: Year,
    month: u32,
}

impl YearMonth {
    /// Creates a year/month pair.
    fn new(year: Year, month: u32) -> Self {
        Self { year, month }
    }

    /// Returns the current local calendar month.
    fn current() -> Self {
        let (year, month) = local_year_month();
        Self::new(year, month)
    }

    /// Builds a `YearMonth` from a commit date string.
    fn from_commit_date(date: &str) -> Option<Self> {
        let (year, month, _) = parse_date(date);
        if year == 0 || month == 0 {
            return None;
        }
        Some(Self::new(year, month))
    }

    /// Advances to the next calendar month.
    fn next_month(self) -> Self {
        if self.month == 12 {
            Self::new(self.year + 1, 1)
        } else {
            Self::new(self.year, self.month + 1)
        }
    }

    /// Formats the first day of this month as `YYYY-MM-01`.
    fn format_month_start(self) -> String {
        format!("{:04}-{:02}-01", self.year, self.month)
    }
}

/// Returns the current local calendar year and month (1–12).
fn local_year_month() -> (Year, u32) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct SystemTime {
            year: u16,
            month: u16,
            day_of_week: u16,
            day: u16,
            hour: u16,
            minute: u16,
            second: u16,
            milliseconds: u16,
        }

        let mut st = SystemTime {
            year: 0,
            month: 0,
            day_of_week: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            milliseconds: 0,
        };

        unsafe extern "system" {
            fn GetLocalTime(lp_system_time: *mut SystemTime);
        }

        unsafe { GetLocalTime(&mut st) };
        (st.year as Year, st.month as u32)
    }

    #[cfg(not(windows))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        #[repr(C)]
        struct Tm {
            sec: i32,
            min: i32,
            hour: i32,
            mday: i32,
            mon: i32,
            year: i32,
            wday: i32,
            yday: i32,
            isdst: i32,
        }

        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs() as i64;
        let mut tm = Tm {
            sec: 0,
            min: 0,
            hour: 0,
            mday: 0,
            mon: 0,
            year: 0,
            wday: 0,
            yday: 0,
            isdst: 0,
        };

        unsafe extern "C" {
            fn localtime_r(time: *const i64, result: *mut Tm) -> *mut Tm;
        }

        unsafe {
            localtime_r(&secs, &mut tm);
        }
        ((tm.year + 1900) as Year, (tm.mon + 1) as u32)
    }
}

/// Lists calendar months strictly between `prev` and `next` (exclusive on both ends).
fn months_between(prev: YearMonth, next: YearMonth) -> Vec<YearMonth> {
    let mut silent = Vec::new();
    let mut cursor = prev.next_month();
    while cursor < next {
        silent.push(cursor);
        cursor = cursor.next_month();
    }
    silent
}

/// Returns sorted unique months that contain at least one commit.
fn occupied_months(commits: &[Commit]) -> Vec<YearMonth> {
    let mut months: Vec<YearMonth> = commits
        .iter()
        .filter_map(|c| YearMonth::from_commit_date(&c.date))
        .collect();
    months.sort();
    months.dedup();
    months
}

/// Returns the earliest commit (by date) that falls in the given month.
fn first_commit_in_month(commits: &[Commit], month: YearMonth) -> Option<&Commit> {
    commits
        .iter()
        .filter(|c| YearMonth::from_commit_date(&c.date) == Some(month))
        .min_by(|a, b| a.date.cmp(&b.date))
}

struct MostActiveMonth {
    year: Year,
    month: usize,
    count: u32,
}

/// Aggregated commit data built in a single pass over the log.
struct CommitStats {
    commits_by_month: HashMap<Year, [u32; 12]>,
    first_date_by_author: HashMap<String, String>,
    born: Option<usize>,
    latest: Option<usize>,
}

/// Parses `YYYY-MM-DD` or `YYYY-MM-DD HH:MM:SS …` (git `%ai`) into `(year, month, day)`.
fn parse_date(date: &str) -> (u32, u32, u32) {
    let date = date.split_whitespace().next().unwrap_or(date);
    let mut parts = date.split('-');
    let year = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let month = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let day = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (year, month, day)
}

/// Parses a commit date into `(year, zero-based month index)` for chart indexing.
fn parse_year_month(date: &str) -> Option<(Year, usize)> {
    let (year, month, _) = parse_date(date);
    if year == 0 || month == 0 {
        return None;
    }
    Some((year, (month as usize).saturating_sub(1)))
}

/// Extracts the calendar year from an event for grouping and sort placement.
fn event_year(event: &Event) -> Option<Year> {
    match event {
        Event::Born { date, .. }
        | Event::Joined { date, .. }
        | Event::Revival { date, .. }
        | Event::Latest { date, .. } => Some(parse_date(date).0),
        Event::Peak { date, .. } => Some(parse_date(&format!("{date}-01")).0),
        Event::Silence { from, .. } => Some(parse_date(from).0),
        Event::EndOfYear { year, .. } => Some(*year),
    }
}

/// Returns the primary sort date for timeline ordering (excluding `EndOfYear`).
fn event_sort_date(event: &Event) -> (u32, u32, u32) {
    match event {
        Event::Born { date, .. }
        | Event::Joined { date, .. }
        | Event::Revival { date, .. }
        | Event::Latest { date, .. } => parse_date(date),
        Event::Silence { from, .. } => parse_date(from),
        Event::Peak { date, .. } => {
            let (year, month, _) = parse_date(&format!("{date}-01"));
            (year, month, 0)
        }
        Event::EndOfYear { .. } => unreachable!("EndOfYear is sorted separately"),
    }
}

/// Computes where a year chart sits on the timeline relative to that year's other events.
fn end_of_year_sort_date(year: Year, events: &[Event]) -> (u32, u32, u32) {
    let born_in_year = events.iter().find_map(|e| match e {
        Event::Born { date, .. } if parse_date(date).0 == year => Some(parse_date(date)),
        _ => None,
    });

    if let Some((y, m, d)) = born_in_year {
        return (y, m, d.saturating_add(1));
    }

    let mut year_events: Vec<&Event> = events
        .iter()
        .filter(|e| {
            event_year(e) == Some(year)
                && !matches!(
                    e,
                    Event::Born { .. } | Event::EndOfYear { .. } | Event::Latest { .. }
                )
        })
        .collect();

    year_events.sort_by(|a, b| {
        event_sort_date(a)
            .cmp(&event_sort_date(b))
            .then_with(|| event_kind(a).cmp(&event_kind(b)))
    });

    match year_events.first() {
        Some(Event::Joined { date, .. }) => {
            let (y, m, d) = parse_date(date);
            (y, m, d.saturating_add(1))
        }
        Some(event) => {
            let (y, m, _) = event_sort_date(event);
            (y, m.saturating_sub(1).max(1), 1)
        }
        None => (year, 12, 31),
    }
}

/// Returns the tie-breaking kind for same-day timeline events.
fn event_kind(event: &Event) -> EventKind {
    match event {
        Event::Joined { .. } => EventKind::Joined,
        Event::Peak { .. } => EventKind::Peak,
        Event::Silence { .. } => EventKind::Silence,
        Event::Revival { .. } => EventKind::Revival,
        Event::EndOfYear { .. } | Event::Born { .. } | Event::Latest { .. } => {
            unreachable!("handled separately")
        }
    }
}

/// Builds a composite sort key for a single timeline event.
fn sort_key(event: &Event, all_events: &[Event]) -> (u32, u32, u32, EventKind) {
    match event {
        Event::EndOfYear { year, .. } => {
            let (y, m, d) = end_of_year_sort_date(*year, all_events);
            (y, m, d, EventKind::EndOfYear)
        }
        _ => {
            let (y, m, d) = event_sort_date(event);
            (y, m, d, event_kind(event))
        }
    }
}

/// Single pass: per-month counts, per-author first date, born/latest indices.
fn collect_commit_stats(commits: &[Commit]) -> CommitStats {
    let mut commits_by_month: HashMap<Year, [u32; 12]> = HashMap::new();
    let mut first_date_by_author: HashMap<String, String> = HashMap::new();
    let mut born: Option<usize> = None;
    let mut latest: Option<usize> = None;

    for (index, commit) in commits.iter().enumerate() {
        let Some((year, month)) = parse_year_month(&commit.date) else {
            continue;
        };

        born = match born {
            None => Some(index),
            Some(prev) if commit.date < commits[prev].date => Some(index),
            other => other,
        };

        latest = match latest {
            None => Some(index),
            Some(prev) if commit.date > commits[prev].date => Some(index),
            other => other,
        };

        let chart = commits_by_month.entry(year).or_insert([0; 12]);
        chart[month] += 1;

        first_date_by_author
            .entry(commit.author.clone())
            .and_modify(|date| {
                if commit.date < *date {
                    *date = commit.date.clone();
                }
            })
            .or_insert(commit.date.clone());
    }

    CommitStats {
        commits_by_month,
        first_date_by_author,
        born,
        latest,
    }
}

/// Builds `Joined` events from each author's earliest commit date.
fn build_joined_events(commits: &[Commit], stats: &CommitStats) -> Vec<Event> {
    let born_author = stats.born.map(|i| commits[i].author.as_str());

    stats
        .first_date_by_author
        .iter()
        .filter(|(author, _)| born_author != Some(author.as_str()))
        .map(|(author, date)| Event::Joined {
            date: date.clone(),
            author: author.clone(),
        })
        .collect()
}

/// Builds `EndOfYear` events with per-month commit charts.
fn build_end_of_year_events(stats: &CommitStats) -> Vec<Event> {
    stats
        .commits_by_month
        .iter()
        .map(|(year, chart)| Event::EndOfYear {
            year: *year,
            chart: *chart,
        })
        .collect()
}

/// Finds the globally most active month and returns a `Peak` event, if any commits exist.
fn build_peak_event(stats: &CommitStats) -> Option<Event> {
    let mut most_active_month: Option<MostActiveMonth> = None;

    for (year, chart) in &stats.commits_by_month {
        for (month_index, &count) in chart.iter().enumerate() {
            if count == 0 {
                continue;
            }
            most_active_month = match most_active_month {
                None => Some(MostActiveMonth {
                    year: *year,
                    month: month_index,
                    count,
                }),
                Some(ref best) if count > best.count => Some(MostActiveMonth {
                    year: *year,
                    month: month_index,
                    count,
                }),
                other => other,
            };
        }
    }

    most_active_month.map(|MostActiveMonth { year, month, count }| Event::Peak {
        date: format!("{:04}-{:02}", year, month + 1),
        count,
    })
}

/// Builds a `Born` event from the earliest commit in the log.
fn build_born_event(commits: &[Commit], stats: &CommitStats) -> Option<Event> {
    stats.born.map(|index| {
        let commit = &commits[index];
        Event::Born {
            date: commit.date.clone(),
            message: commit.message.clone(),
            author: commit.author.clone(),
        }
    })
}

/// Builds a `Latest` event from the newest commit, unless it is the same commit as `Born`.
fn build_latest_event(commits: &[Commit], stats: &CommitStats) -> Option<Event> {
    let latest_index = stats.latest?;
    if stats.born == Some(latest_index) {
        return None;
    }
    let commit = &commits[latest_index];
    Some(Event::Latest {
        date: commit.date.clone(),
        message: commit.message.clone(),
        author: commit.author.clone(),
    })
}

/// Detects silent months between commit activity and emits `Silence` / `Revival` pairs.
///
/// Each `Silence` is always followed by a `Revival` for the next month that has commits.
/// Gaps from the last commit up to "now" are not shown (no orphan silence).
fn build_silence_revival_events(commits: &[Commit]) -> Vec<Event> {
    let now = YearMonth::current();
    build_silence_revival_events_until(commits, now.next_month())
}

/// Same as [`build_silence_revival_events`] but with an explicit exclusive upper bound (reserved for tests).
fn build_silence_revival_events_until(
    commits: &[Commit],
    _until_exclusive: YearMonth,
) -> Vec<Event> {
    let months = occupied_months(commits);
    if months.is_empty() {
        return Vec::new();
    }

    let mut events = Vec::new();

    for window in months.windows(2) {
        let prev = window[0];
        let next = window[1];
        let silent = months_between(prev, next);
        if silent.is_empty() {
            continue;
        }
        push_silence_and_revival(commits, &mut events, &silent, Some(next));
    }

    events
}

/// Appends `Silence` and, when `revival_month` is set, the first commit in that month.
fn push_silence_and_revival(
    commits: &[Commit],
    events: &mut Vec<Event>,
    silent: &[YearMonth],
    revival_month: Option<YearMonth>,
) {
    push_silence_only(events, silent);

    if let Some(month) = revival_month {
        if let Some(commit) = first_commit_in_month(commits, month) {
            events.push(Event::Revival {
                date: commit.date.clone(),
                msg: commit.message.clone(),
            });
        }
    }
}

/// Appends a `Silence` event covering the given empty months.
fn push_silence_only(events: &mut Vec<Event>, silent: &[YearMonth]) {
    if silent.is_empty() {
        return;
    }
    events.push(Event::Silence {
        from: silent[0].format_month_start(),
        months: silent.len() as u32,
    });
}

/// Orchestrates event generation from parsed commits.
pub fn generate_events_from_commits(commits: &[Commit]) -> Vec<Event> {
    let stats = collect_commit_stats(commits);
    let mut events = Vec::new();

    events.extend(build_joined_events(commits, &stats));
    events.extend(build_end_of_year_events(&stats));
    if let Some(peak) = build_peak_event(&stats) {
        events.push(peak);
    }
    if let Some(born) = build_born_event(commits, &stats) {
        events.push(born);
    }
    events.extend(build_silence_revival_events(commits));
    if let Some(latest) = build_latest_event(commits, &stats) {
        events.push(latest);
    }

    events
}

/// Sorts events for timeline display: Born first, Latest last, everything else by date.
pub fn sort_events(events: Vec<Event>) -> Vec<Event> {
    let mut born = None;
    let mut latest = None;
    let mut rest = Vec::new();

    for event in events {
        match event {
            Event::Born { .. } => born = Some(event),
            Event::Latest { .. } => latest = Some(event),
            other => rest.push(other),
        }
    }

    let all_for_keys = {
        let mut snapshot: Vec<Event> = Vec::new();
        if let Some(ref b) = born {
            snapshot.push(b.clone());
        }
        snapshot.extend(rest.iter().cloned());
        if let Some(ref l) = latest {
            snapshot.push(l.clone());
        }
        snapshot
    };

    rest.sort_by(|a, b| {
        let key_a = sort_key(a, &all_for_keys);
        let key_b = sort_key(b, &all_for_keys);
        key_a.cmp(&key_b)
    });

    let mut sorted = Vec::new();
    if let Some(b) = born {
        sorted.push(b);
    }
    sorted.extend(rest);
    if let Some(l) = latest {
        sorted.push(l);
    }
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(date: &str, author: &str, message: &str) -> Commit {
        Commit::new("hash".into(), author.into(), date.into(), message.into())
    }

    fn sort_keys(events: &[Event]) -> Vec<String> {
        sort_events(events.to_vec())
            .iter()
            .map(|e| match e {
                Event::Born { .. } => "born".to_string(),
                Event::Joined { .. } => "joined".to_string(),
                Event::Peak { .. } => "peak".to_string(),
                Event::Silence { .. } => "silence".to_string(),
                Event::Revival { .. } => "revival".to_string(),
                Event::Latest { .. } => "latest".to_string(),
                Event::EndOfYear { year, .. } => format!("year-{year}"),
            })
            .collect()
    }

    #[test]
    fn timeline_order_matches_schema() {
        let events = vec![
            Event::Latest {
                date: "2024-11-15".into(),
                message: "Fix auth bug".into(),
                author: "charlie@corp.com".into(),
            },
            Event::EndOfYear {
                year: 2022,
                chart: [0; 12],
            },
            Event::EndOfYear {
                year: 2021,
                chart: [0; 12],
            },
            Event::Joined {
                date: "2021-03-10".into(),
                author: "bob@corp.com".into(),
            },
            Event::Revival {
                date: "2020-12-01".into(),
                msg: "Back to it".into(),
            },
            Event::Silence {
                from: "2020-08-01".into(),
                months: 4,
            },
            Event::Peak {
                date: "2020-02".into(),
                count: 47,
            },
            Event::EndOfYear {
                year: 2020,
                chart: [0; 12],
            },
            Event::Joined {
                date: "2019-06-15".into(),
                author: "carol@corp.com".into(),
            },
            Event::EndOfYear {
                year: 2019,
                chart: [0; 12],
            },
            Event::Born {
                date: "2019-03-01".into(),
                message: "Initial commit".into(),
                author: "alice@corp.com".into(),
            },
        ];

        assert_eq!(
            sort_keys(&events),
            vec![
                "born",
                "year-2019",
                "joined",
                "year-2020",
                "peak",
                "silence",
                "revival",
                "joined",
                "year-2021",
                "year-2022",
                "latest",
            ]
        );
    }

    #[test]
    fn generates_latest_commit() {
        let commits = vec![
            commit("2019-03-01", "alice@corp.com", "Initial commit"),
            commit("2024-11-15", "charlie@corp.com", "Fix auth bug"),
        ];

        let events = sort_events(generate_events_from_commits(&commits));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Latest {
                message,
                author,
                ..
            } if message == "Fix auth bug" && author == "charlie@corp.com"
        )));
        assert!(matches!(events.last(), Some(Event::Latest { .. })));
    }

    #[test]
    fn born_and_latest_same_commit_only_born() {
        let commits = vec![commit("2019-03-01", "alice@corp.com", "Initial commit")];
        let events = generate_events_from_commits(&commits);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Latest { .. }))
                .count(),
            0
        );
    }

    #[test]
    fn silence_and_revival_between_commits() {
        let commits = vec![
            commit("2020-07-15", "alice@corp.com", "Summer work"),
            commit("2020-12-01", "alice@corp.com", "Back to it"),
        ];

        let events = build_silence_revival_events_until(&commits, YearMonth::new(2020, 12));

        assert!(events.iter().any(|e| matches!(
            e,
            Event::Silence { from, months } if from == "2020-08-01" && *months == 4
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Revival { date, msg } if date == "2020-12-01" && msg == "Back to it"
        )));
    }

    #[test]
    fn no_trailing_silence_without_revival() {
        let commits = vec![commit("2020-07-15", "alice@corp.com", "Last activity")];

        let events = build_silence_revival_events_until(&commits, YearMonth::new(2020, 11));

        assert!(!events.iter().any(|e| matches!(e, Event::Silence { .. })));
        assert!(!events.iter().any(|e| matches!(e, Event::Revival { .. })));
    }

    #[test]
    fn multiple_silences() {
        let commits = vec![
            commit("2020-01-10", "a@corp.com", "Start"),
            commit("2020-04-10", "a@corp.com", "Spring"),
            commit("2020-09-10", "a@corp.com", "Fall"),
        ];

        let events = build_silence_revival_events_until(&commits, YearMonth::new(2020, 9));

        let silences: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, Event::Silence { .. }))
            .collect();
        let revivals: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, Event::Revival { .. }))
            .collect();

        assert_eq!(silences.len(), 2);
        assert_eq!(revivals.len(), 2);
        assert!(matches!(
            silences[0],
            Event::Silence { from, months } if from == "2020-02-01" && *months == 2
        ));
        assert!(matches!(
            silences[1],
            Event::Silence { from, months } if from == "2020-05-01" && *months == 4
        ));
    }

    #[test]
    fn generates_silence_revival_in_timeline() {
        let commits = vec![
            commit("2019-03-01", "alice@corp.com", "Initial commit"),
            commit("2020-02-15", "alice@corp.com", "Peak month work"),
            commit("2020-07-15", "alice@corp.com", "Mid year"),
            commit("2020-12-01", "alice@corp.com", "Back to it"),
            commit("2021-03-10", "bob@corp.com", "Bob joins"),
            commit("2024-11-15", "charlie@corp.com", "Fix auth bug"),
        ];

        let keys = sort_keys(&generate_events_from_commits(&commits));

        let silence_pos = keys.iter().position(|k| k == "silence").expect("silence");
        let revival_pos = keys.iter().position(|k| k == "revival").expect("revival");
        assert!(silence_pos < revival_pos);
        assert!(keys.contains(&"silence".to_string()));
        assert!(keys.contains(&"revival".to_string()));
    }
}
