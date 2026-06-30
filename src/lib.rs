use std::convert::TryFrom;

mod analyzer;
mod git;
mod parser;
mod render;

use render::{ColorOutput, GlyphSet};

pub enum Command {
    Timeline,
    Who,
    Churn,
    Help,
}

/// User-facing command names (keep in sync with `help_text` and `Command::try_from`).
const COMMAND_NAMES: &[&str] = &["timeline", "who", "churn", "help"];

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Suggests the closest command name for a typo (git-style "did you mean").
fn suggest_command(typo: &str) -> Option<&'static str> {
    let typo = typo.to_ascii_lowercase();
    let mut best: Option<(&str, usize)> = None;

    for &name in COMMAND_NAMES {
        let distance = levenshtein_distance(&typo, name);
        let threshold = name.len().saturating_sub(1).max(1);
        if distance > threshold {
            continue;
        }
        match best {
            None => best = Some((name, distance)),
            Some((_, best_dist)) if distance < best_dist => best = Some((name, distance)),
            _ => {}
        }
    }

    best.map(|(name, _)| name)
}

fn unknown_command_message(typo: &str) -> String {
    let program = program_invocation();
    let mut message = format!("unknown command '{typo}'. Run `{program} help` for usage.");
    if let Some(similar) = suggest_command(typo) {
        message.push_str("\n\nThe most similar command is\n\t");
        message.push_str(similar);
    }
    message
}

impl TryFrom<&str> for Command {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "timeline" => Ok(Command::Timeline),
            "who" => Ok(Command::Who),
            "churn" => Ok(Command::Churn),
            "help" | "--help" | "-h" => Ok(Command::Help),
            other => Err(unknown_command_message(other)),
        }
    }
}

pub fn program_invocation() -> String {
    std::env::args()
        .next()
        .as_deref()
        .and_then(|p| std::path::Path::new(p).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("grin")
        .to_string()
}

pub(crate) fn help_text(program: &str) -> String {
    format!(
        "{program} — git repository analytics\n\n\
Commands:\n\
  timeline    activity timeline from git log\n\
  who         top contributors\n\
  churn       files with the most changes\n\
  help        show this message\n\n\
Flags (before or after the command):\n\
  --limit N     max rows (default: 5)\n\
  --ext LIST    churn only: comma-separated file extensions (e.g. ts,tsx)\n\
  --no-color    disable ANSI colors; also env NO_COLOR\n\
  --ascii       ASCII symbols instead of Unicode; also env GRIN_ASCII\n"
    )
}

pub struct Config {
    pub command: Command,
    pub limit: usize,
    pub churn_extensions: Option<Vec<String>>,
    pub color: ColorOutput,
    pub glyphs: GlyphSet,
}

fn normalize_ext_token(token: &str) -> Result<String, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("--ext has an empty entry (check commas/spaces).".into());
    }
    let stripped = token.strip_prefix('.').unwrap_or(token);
    if stripped.is_empty() {
        return Err("--ext has an invalid empty extension.".into());
    }
    Ok(stripped.to_ascii_lowercase())
}

fn parse_extensions_list(raw: &str) -> Result<Vec<String>, String> {
    let tokens: Vec<Result<String, String>> = raw.split(',').map(normalize_ext_token).collect();
    let mut out = Vec::new();
    for t in tokens {
        out.push(t?);
    }
    if out.is_empty() {
        return Err("--ext needs at least one extension (e.g. ts,tsx).".into());
    }
    Ok(out)
}

/// Removes global flags from argv tail; returns cleaned args and flag presence.
fn strip_global_flags(tail: &[String]) -> (Vec<String>, bool, bool) {
    let mut no_color = false;
    let mut ascii = false;
    let mut out = Vec::with_capacity(tail.len());
    for arg in tail {
        match arg.as_str() {
            "--no-color" | "-C" => no_color = true,
            "--ascii" => ascii = true,
            other => out.push(other.to_string()),
        }
    }
    (out, no_color, ascii)
}

pub(crate) fn resolve_color(no_color_flag: bool) -> ColorOutput {
    if no_color_flag || std::env::var_os("NO_COLOR").is_some() {
        ColorOutput::Never
    } else {
        ColorOutput::AutoTerminal
    }
}

pub(crate) fn resolve_glyphs(ascii_flag: bool) -> GlyphSet {
    let env_ascii = std::env::var("GRIN_ASCII")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if ascii_flag || env_ascii {
        GlyphSet::Ascii
    } else {
        GlyphSet::Unicode
    }
}

fn parse_flags(command: &Command, args: &[String]) -> Result<(usize, Option<Vec<String>>), String> {
    let mut limit = 5usize;
    let mut churn_extensions: Option<Vec<String>> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--limit" => {
                i += 1;
                let value = args.get(i).ok_or("missing value for `--limit`")?;
                limit = value
                    .parse()
                    .map_err(|_| "`--limit` must be a positive integer")?;
                if limit == 0 {
                    return Err("`--limit` must be at least 1".into());
                }
                i += 1;
            }
            "--ext" => {
                if !matches!(command, Command::Churn) {
                    return Err(
                        "`--ext` is only valid with the `churn` command. Run `help` for usage."
                            .into(),
                    );
                }
                i += 1;
                let value = args.get(i).ok_or("missing value for `--ext`")?;
                churn_extensions = Some(parse_extensions_list(value)?);
                i += 1;
            }
            "--no-color" | "-C" | "--ascii" => {
                return Err(format!(
                    "global flag `{}` must appear before or after the command name, not mixed with `--limit` / `--ext` values.",
                    args[i]
                ));
            }
            flag => {
                return Err(format!(
                    "unknown argument `{flag}`. Run `{} help` for usage.",
                    program_invocation()
                ));
            }
        }
    }
    Ok((limit, churn_extensions))
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self, String> {
        let default = Self {
            command: Command::Help,
            limit: 5,
            churn_extensions: None,
            color: resolve_color(false),
            glyphs: resolve_glyphs(false),
        };

        if args.len() < 2 {
            return Ok(default);
        }

        let (tail, no_color_flag, ascii_flag) = strip_global_flags(&args[1..]);
        let color = resolve_color(no_color_flag);
        let glyphs = resolve_glyphs(ascii_flag);

        if tail.is_empty() {
            return Ok(Self {
                color,
                glyphs,
                ..default
            });
        }

        let mut full_args = vec![args[0].clone()];
        full_args.extend(tail);

        if full_args.len() < 2 {
            return Ok(Self {
                color,
                glyphs,
                ..default
            });
        }

        let command = Command::try_from(full_args[1].as_str())?;

        if matches!(command, Command::Help) {
            return Ok(Self {
                command,
                limit: 5,
                churn_extensions: None,
                color,
                glyphs,
            });
        }

        let (limit, churn_extensions) = parse_flags(&command, &full_args)?;

        Ok(Self {
            command,
            limit,
            churn_extensions,
            color,
            glyphs,
        })
    }
}

pub fn run(config: Config) {
    render::init_render(config.color, config.glyphs);

    if matches!(config.command, Command::Help) {
        let program = program_invocation();
        print!("{}", help_text(&program));
        return;
    }

    let repo_name = git::get_repo_name().unwrap_or_else(|_| "repository".into());

    match config.command {
        Command::Timeline => {
            let log = git::get_log().unwrap();
            let commits = parser::parse_log(&log).unwrap();
            let events = analyzer::sort_events(analyzer::generate_events_from_commits(&commits));
            render::print_timeline(&render::Timeline {
                repo_name: &repo_name,
                commits: &commits,
                events: &events,
            });
        }
        Command::Who => {
            let log = git::get_log().unwrap();
            let commits = parser::parse_log(&log).unwrap();
            let contributors = analyzer::contributor_stats(&commits, config.limit);
            render::print_who(&render::WhoReport {
                repo_name: &repo_name,
                contributors: &contributors,
            });
        }
        Command::Churn => {
            let log = git::get_log_with_files().unwrap();
            let touches = parser::parse_file_touches(&log).unwrap();
            let ext_filter = config.churn_extensions.as_deref();
            let (files, summary) = analyzer::file_churn_stats(&touches, config.limit, ext_filter);
            let extensions_note = config.churn_extensions.as_ref().map(|list| list.join(", "));
            render::print_churn(&render::ChurnReport {
                repo_name: &repo_name,
                files: &files,
                summary: &summary,
                extensions_filter: extensions_note.as_deref(),
            });
        }
        Command::Help => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_limit() {
        let args = vec!["grin".into(), "who".into()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.limit, 5);
        assert!(config.churn_extensions.is_none());
        if std::env::var_os("NO_COLOR").is_none() {
            assert_eq!(config.color, ColorOutput::AutoTerminal);
        }
        if std::env::var("GRIN_ASCII")
            .map(|v| v.is_empty())
            .unwrap_or(true)
        {
            assert_eq!(config.glyphs, GlyphSet::Unicode);
        }
    }

    #[test]
    fn config_parses_limit_flag() {
        let args = vec!["grin".into(), "churn".into(), "--limit".into(), "10".into()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.limit, 10);
    }

    #[test]
    fn config_rejects_unknown_argument() {
        let args = vec!["grin".into(), "who".into(), "10".into()];
        assert!(Config::build(&args).is_err());
        let err = Config::build(&args).err().unwrap();
        assert!(err.contains("help"));
    }

    #[test]
    fn no_args_means_help() {
        let args = vec!["grin".into()];
        let config = Config::build(&args).unwrap();
        assert!(matches!(config.command, Command::Help));
    }

    #[test]
    fn help_command() {
        let args = vec!["grin".into(), "help".into()];
        let config = Config::build(&args).unwrap();
        assert!(matches!(config.command, Command::Help));
    }

    #[test]
    fn double_dash_help_synonym() {
        let args = vec!["grin".into(), "--help".into()];
        let config = Config::build(&args).unwrap();
        assert!(matches!(config.command, Command::Help));
    }

    #[test]
    fn churn_parses_extensions() {
        let args = vec![
            "grin".into(),
            "churn".into(),
            "--ext".into(),
            "ts, .tsx".into(),
        ];
        let config = Config::build(&args).unwrap();
        assert_eq!(
            config.churn_extensions,
            Some(vec!["ts".into(), "tsx".into()])
        );
    }

    #[test]
    fn ext_not_allowed_for_who() {
        let args = vec!["grin".into(), "who".into(), "--ext".into(), "rs".into()];
        assert!(Config::build(&args).is_err());
    }

    #[test]
    fn no_color_flag_before_command() {
        let args = vec!["grin".into(), "--no-color".into(), "who".into()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.color, ColorOutput::Never);
        assert!(matches!(config.command, Command::Who));
    }

    #[test]
    fn no_color_flag_after_command() {
        let args = vec!["grin".into(), "timeline".into(), "--no-color".into()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.color, ColorOutput::Never);
        assert!(matches!(config.command, Command::Timeline));
    }

    #[test]
    fn ascii_flag_sets_glyph_set() {
        let args = vec!["grin".into(), "--ascii".into(), "who".into()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.glyphs, GlyphSet::Ascii);
    }

    #[test]
    fn resolve_color_respects_no_color_env() {
        let key = "NO_COLOR";
        let previous = std::env::var_os(key);
        // SAFETY: restored before test returns.
        unsafe { std::env::set_var(key, "1") };
        assert_eq!(resolve_color(false), ColorOutput::Never);
        match previous {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
    }

    #[test]
    fn no_color_flag_overrides_tty_auto() {
        let args = vec!["grin".into(), "who".into(), "--no-color".into()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.color, ColorOutput::Never);
    }

    #[test]
    fn help_text_documents_global_flags() {
        let text = help_text("grin");
        assert!(text.contains("--no-color"));
        assert!(text.contains("--ascii"));
        assert!(text.contains("NO_COLOR"));
        assert!(text.contains("GRIN_ASCII"));
    }

    #[test]
    fn unknown_command_suggests_similar_name() {
        let args = vec!["grin".into(), "timline".into()];
        let err = Config::build(&args).err().unwrap();
        assert!(err.contains("unknown command 'timline'"));
        assert!(err.contains("The most similar command is"));
        assert!(err.contains("timeline"));
    }

    #[test]
    fn unknown_command_no_suggestion_when_too_different() {
        let args = vec!["grin".into(), "zzzzzz".into()];
        let err = Config::build(&args).err().unwrap();
        assert!(err.contains("unknown command 'zzzzzz'"));
        assert!(!err.contains("The most similar command is"));
    }

    #[test]
    fn suggest_command_matches_git_style_typos() {
        assert_eq!(suggest_command("churj"), Some("churn"));
        assert_eq!(suggest_command("wo"), Some("who"));
    }
}
