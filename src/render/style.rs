use std::io::{self, IsTerminal};

/// Whether ANSI styling may be emitted.
#[allow(dead_code)] // `Never` used in tests and for explicit opt-out
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorOutput {
    Never,
    AutoTerminal,
}

impl ColorOutput {
    pub(crate) fn is_on(self) -> bool {
        match self {
            ColorOutput::Never => false,
            ColorOutput::AutoTerminal => io::stdout().is_terminal(),
        }
    }
}

pub(crate) fn use_color() -> bool {
    ColorOutput::AutoTerminal.is_on()
}

pub(crate) fn style(text: &str, code: &str) -> String {
    if ColorOutput::AutoTerminal.is_on() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub(crate) fn bold(text: &str) -> String {
    style(text, "1")
}

pub(crate) fn cyan(text: &str) -> String {
    style(text, "36")
}

pub(crate) fn blue(text: &str) -> String {
    style(text, "34")
}

pub(crate) fn green(text: &str) -> String {
    style(text, "32")
}

pub(crate) fn magenta(text: &str) -> String {
    style(text, "35")
}

/// Normal red (churn bars, etc.); distinct from [`dim_red`].
pub(crate) fn red(text: &str) -> String {
    style(text, "31")
}

pub(crate) fn yellow(text: &str) -> String {
    style(text, "33")
}

pub(crate) fn dim_red(text: &str) -> String {
    style(text, "2;31")
}
