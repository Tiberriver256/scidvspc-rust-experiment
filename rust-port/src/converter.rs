//! High-level API for SCID to PGN conversion

use crate::game::Game;
use crate::scid_codec::{ByteBuffer, DecodeError, decode_game, DECODE_ALL};
use crate::pgn::{PgnWriter, PgnOptions};
use crate::pgn::writer::PgnError;

/// Convert a SCID binary game to PGN string
pub fn scid_to_pgn(data: &[u8], options: PgnOptions) -> Result<String, ConversionError> {
    // Decode from SCID format
    let mut buf = ByteBuffer::new(data.to_vec());
    let game = decode_game(&mut buf, DECODE_ALL)?;
    
    // Write to PGN
    let writer = PgnWriter::new(options);
    let pgn = writer.write(&game)?;
    
    Ok(pgn)
}

/// Errors that can occur during conversion
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Failed to decode SCID format: {0}")]
    DecodeError(#[from] DecodeError),
    
    #[error("Failed to generate PGN: {0}")]
    PgnError(#[from] PgnError),
}

/// Builder for conversion with options
pub struct Converter {
    pgn_options: PgnOptions,
}

impl Converter {
    pub fn new() -> Self {
        Converter {
            pgn_options: PgnOptions::default(),
        }
    }
    
    pub fn with_options(mut self, options: PgnOptions) -> Self {
        self.pgn_options = options;
        self
    }
    
    pub fn include_comments(mut self, include: bool) -> Self {
        self.pgn_options.include_comments = include;
        self
    }
    
    pub fn include_variations(mut self, include: bool) -> Self {
        self.pgn_options.include_variations = include;
        self
    }
    
    pub fn convert(&self, data: &[u8]) -> Result<String, ConversionError> {
        scid_to_pgn(data, self.pgn_options)
    }
    
    pub fn convert_to_game(&self, data: &[u8]) -> Result<Game, ConversionError> {
        let mut buf = ByteBuffer::new(data.to_vec());
        Ok(decode_game(&mut buf, DECODE_ALL)?)
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_builder() {
        let converter = Converter::new()
            .include_comments(false)
            .include_variations(false);
        
        assert!(!converter.pgn_options.include_comments);
        assert!(!converter.pgn_options.include_variations);
    }
}
