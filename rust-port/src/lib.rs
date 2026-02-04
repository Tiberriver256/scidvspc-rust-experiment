//! SCID PGN Converter - Rust Implementation
//! 
//! This is a Rust port of the SCID database to PGN conversion functionality.
//! It aims for behavioral equivalence with the C++ implementation while
//! leveraging Rust's safety and modern language features.

pub mod types;
pub mod game;
pub mod position;
pub mod pgn;
pub mod scid_codec;
pub mod converter;

pub use game::Game;
pub use pgn::PgnWriter;
pub use converter::{Converter, scid_to_pgn};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
