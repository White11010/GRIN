use std::convert::TryFrom;

mod analyzer;
mod git;
mod parser;
mod render;

pub enum Command {
    Timeline,
    Who,
    Churn,
    Help,
}

impl TryFrom<&str> for Command {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "timeline" => Ok(Command::Timeline),
            "who" => Ok(Command::Who),
            "churn" => Ok(Command::Churn),
            "help" | "--help" | "-h" => Ok(Command::Help),
            other => Err(format!(
                "unknown command '{other}'. Run `{} help` for usage.",
                program_invocation()
            )),
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
Flags (after the command):\n\
  --limit N   max rows (default: 5)\n\
  --ext LIST  churn only: comma-separated file extensions (e.g. ts,tsx)\n"
    )
}

pub struct Config {
    pub command: Command,
    pub limit: usize,
    pub churn_extensions: Option<Vec<String>>,
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
        if args.len() < 2 {
            return Ok(Self {
                command: Command::Help,
                limit: 5,
                churn_extensions: None,
            });
        }

        let command = Command::try_from(args[1].as_str())?;

        if matches!(command, Command::Help) {
            return Ok(Self {
                command,
                limit: 5,
                churn_extensions: None,
            });
        }

        let (limit, churn_extensions) = parse_flags(&command, args)?;

        Ok(Self {
            command,
            limit,
            churn_extensions,
        })
    }
}

pub fn run(config: Config) {
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
}
