use super::glyphs;
use super::style::{ColorOutput, current_color, green, red};

pub(crate) const WHO_BAR_WIDTH: usize = 12;
pub(crate) const CHURN_BAR_WIDTH: usize = 20;

/// Fill color for the proportional bar (empty segments stay unstyled).
/// Variants match [`super::style`] helpers (`green` / `red`).
#[allow(dead_code)] // `Plain` used in tests; API for callers that skip bar color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RatioBarColor {
    Plain,
    Green,
    Red,
}

/// Renders a proportional bar with filled and empty block characters.
/// Non-zero `value` always shows at least one filled cell when `max > 0`.
pub(crate) fn format_ratio_bar(
    value: u32,
    max: u32,
    width: usize,
    color: RatioBarColor,
    output: ColorOutput,
) -> String {
    let mut filled = if max == 0 {
        0
    } else {
        ((value as f64 / max as f64) * width as f64).round() as usize
    };
    if value > 0 && max > 0 {
        filled = filled.max(1);
    }
    let filled = filled.min(width);
    let filled_ch = glyphs::bar_filled();
    let empty_ch = glyphs::bar_empty();
    let filled_str: String = std::iter::repeat_n(filled_ch, filled).collect();
    let empty_str: String = std::iter::repeat_n(empty_ch, width - filled).collect();

    match (color, output.is_on()) {
        (RatioBarColor::Plain, _) | (_, false) => format!("{filled_str}{empty_str}"),
        (RatioBarColor::Green, true) => format!("{}{}", green(&filled_str), empty_str),
        (RatioBarColor::Red, true) => format!("{}{}", red(&filled_str), empty_str),
    }
}

/// Renders a bar using the process-wide color mode from [`current_color`].
pub(crate) fn format_ratio_bar_current(
    value: u32,
    max: u32,
    width: usize,
    color: RatioBarColor,
) -> String {
    format_ratio_bar(value, max, width, color, current_color())
}
