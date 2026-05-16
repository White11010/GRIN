use std::sync::RwLock;

/// Terminal glyph set for box-drawing and block characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphSet {
    Unicode,
    Ascii,
}

static GLYPHS: RwLock<Option<GlyphSet>> = RwLock::new(None);

/// Sets the global glyph set for this process (call once at startup from `run`).
pub fn init_glyphs(set: GlyphSet) {
    *GLYPHS.write().expect("glyph lock") = Some(set);
}

fn active() -> GlyphSet {
    (*GLYPHS.read().expect("glyph lock")).unwrap_or(GlyphSet::Unicode)
}

/// Sparkline character for activity level `0..=8`.
pub fn sparkline_char(level: usize) -> char {
    const UNICODE: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    const ASCII: [char; 9] = [' ', '.', ':', '-', '=', '+', '*', '#', '#'];
    let table = match active() {
        GlyphSet::Unicode => &UNICODE,
        GlyphSet::Ascii => &ASCII,
    };
    table[level.min(8)]
}

pub fn rule_char() -> char {
    match active() {
        GlyphSet::Unicode => '─',
        GlyphSet::Ascii => '-',
    }
}

pub fn spine_vertical() -> char {
    match active() {
        GlyphSet::Unicode => '│',
        GlyphSet::Ascii => '|',
    }
}

pub fn spine_last() -> char {
    match active() {
        GlyphSet::Unicode => '·',
        GlyphSet::Ascii => '.',
    }
}

pub fn year_sparkline_sep() -> char {
    spine_vertical()
}

/// Milestone marker prefix (e.g. before `born`, `peak`).
pub fn milestone() -> &'static str {
    match active() {
        GlyphSet::Unicode => "◆",
        GlyphSet::Ascii => "*",
    }
}

pub fn silence_dash() -> char {
    match active() {
        GlyphSet::Unicode => '╌',
        GlyphSet::Ascii => '-',
    }
}

/// Prefix before a silence label (`  ╌╌ label ` or `  -- label `).
pub fn silence_head_prefix() -> String {
    let dash = silence_dash();
    match active() {
        GlyphSet::Unicode => format!("  {dash}{dash} "),
        GlyphSet::Ascii => "  -- ".to_string(),
    }
}

pub fn bar_filled() -> char {
    match active() {
        GlyphSet::Unicode => '█',
        GlyphSet::Ascii => '#',
    }
}

pub fn bar_empty() -> char {
    match active() {
        GlyphSet::Unicode => '░',
        GlyphSet::Ascii => '.',
    }
}

/// Section separator in headers and footers.
pub fn separator_dot() -> &'static str {
    match active() {
        GlyphSet::Unicode => "·",
        GlyphSet::Ascii => ".",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_sparkline_uses_ascii_chars() {
        init_glyphs(GlyphSet::Ascii);
        assert_eq!(sparkline_char(0), ' ');
        assert_eq!(sparkline_char(8), '#');
        assert!(!format!("{}", sparkline_char(4)).contains('█'));
    }

    #[test]
    fn ascii_bars_use_hash_and_dot() {
        init_glyphs(GlyphSet::Ascii);
        assert_eq!(bar_filled(), '#');
        assert_eq!(bar_empty(), '.');
    }
}
