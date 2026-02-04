//! Main PGN writer

use crate::game::Game;
use super::{PgnOptions, moves::MoveFormatter};
use super::tags::TagWriter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PgnError {
    #[error("Invalid game state: {0}")]
    InvalidGameState(String),
}

pub struct PgnWriter {
    options: PgnOptions,
}

impl PgnWriter {
    pub fn new(options: PgnOptions) -> Self {
        PgnWriter { options }
    }
    
    pub fn write(&self, game: &Game) -> Result<String, PgnError> {
        let mut output = String::new();
        
        // Write tags
        let tags = TagWriter::write_tags(
            game, 
            self.options.format, 
            self.options.short_header
        );
        output.push_str(&tags);
        output.push('\n');
        
        // Write moves if present
        if !game.moves.is_empty() {
            let formatter = MoveFormatter::new(self.options.clone());
            let moves_str = formatter.format_moves(game);
            output.push_str(&moves_str);
            output.push(' ');
        }
        
        // Write result
        output.push_str(&format!("{}\n", game.result));
        
        Ok(output)
    }
}

impl Default for PgnWriter {
    fn default() -> Self {
        Self::new(PgnOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GameResult;

    #[test]
    fn test_write_simple_game() {
        let mut game = Game::new();
        game.event = "Test Event".to_string();
        game.site = "Test Site".to_string();
        game.white = "Player 1".to_string();
        game.black = "Player 2".to_string();
        game.result = GameResult::White;
        
        let writer = PgnWriter::default();
        let pgn = writer.write(&game).unwrap();
        
        assert!(pgn.contains("[Event \"Test Event\"]"));
        assert!(pgn.contains("[White \"Player 1\"]"));
        assert!(pgn.contains("1-0"));
    }
}
