//! Terminal rendering for timeline, who, and churn reports.

mod churn;
mod common;
pub mod glyphs;
mod style;
mod timeline;
mod who;

pub use glyphs::GlyphSet;
pub use style::ColorOutput;

pub use churn::{ChurnReport, print_churn};
pub use timeline::{Timeline, print_timeline};
pub use who::{WhoReport, print_who};

/// Initializes global render options (color and glyphs). Call once at startup.
pub fn init_render(color: ColorOutput, glyphs: GlyphSet) {
    style::init_color(color);
    glyphs::init_glyphs(glyphs);
}
