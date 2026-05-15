use super::style::{ColorOutput, green, red};

pub(crate) const WHO_BAR_WIDTH: usize = 12;
pub(crate) const CHURN_BAR_WIDTH: usize = 20;

/// Fill color for the proportional bar (`░` segments stay unstyled).
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
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(width - filled);

    match (color, output.is_on()) {
        (RatioBarColor::Plain, _) | (_, false) => format!("{filled_str}{empty_str}"),
        (RatioBarColor::Green, true) => format!("{}{}", green(&filled_str), empty_str),
        (RatioBarColor::Red, true) => format!("{}{}", red(&filled_str), empty_str),
    }
}
