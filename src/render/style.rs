use std::io::{self, IsTerminal};
use std::sync::RwLock;

/// Whether ANSI styling may be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOutput {
    Never,
    AutoTerminal,
}

static COLOR: RwLock<Option<ColorOutput>> = RwLock::new(None);

/// Sets the global color mode for this process (call once at startup from `run`).
pub(crate) fn init_color(mode: ColorOutput) {
    *COLOR.write().expect("color lock") = Some(mode);
}

fn active_color() -> ColorOutput {
    (*COLOR.read().expect("color lock")).unwrap_or(ColorOutput::AutoTerminal)
}

/// Current color mode after [`init_color`].
pub(crate) fn current_color() -> ColorOutput {
    active_color()
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
    current_color().is_on()
}

pub(crate) fn style(text: &str, code: &str) -> String {
    if current_color().is_on() {
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

#[allow(dead_code)]
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
