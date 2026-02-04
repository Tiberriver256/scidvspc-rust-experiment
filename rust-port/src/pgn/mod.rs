//! PGN export functionality

pub mod tags;
pub mod moves;
pub mod writer;

pub use writer::PgnWriter;

/// PGN output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgnFormat {
    Plain,
    Html,
    Latex,
    Color,
}

/// PGN output options
#[derive(Debug, Clone, Copy)]
pub struct PgnOptions {
    pub format: PgnFormat,
    pub include_comments: bool,
    pub include_variations: bool,
    pub include_nags: bool,
    pub short_header: bool,
}

impl Default for PgnOptions {
    fn default() -> Self {
        PgnOptions {
            format: PgnFormat::Plain,
            include_comments: true,
            include_variations: true,
            include_nags: true,
            short_header: false,
        }
    }
}
