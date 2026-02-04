//! Game representation

use crate::types::GameResult;
use crate::position::Position;

/// A node in the move tree
#[derive(Debug, Clone)]
pub struct MoveNode {
    pub move_data: crate::types::Move,
    pub san: String,
    pub comment: Option<String>,
    pub nags: Vec<u8>,
    pub variations: Vec<Vec<MoveNode>>,
}

/// Represents a complete chess game
#[derive(Debug, Clone)]
pub struct Game {
    // Standard PGN tags
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub white: String,
    pub black: String,
    pub result: GameResult,
    
    // Optional tags
    pub white_elo: Option<u16>,
    pub black_elo: Option<u16>,
    pub eco: Option<String>,
    pub event_date: Option<String>,
    
    // Custom tags
    pub extra_tags: Vec<(String, String)>,
    
    // Starting position (None = standard start)
    pub start_position: Option<Position>,
    
    // Moves
    pub moves: Vec<MoveNode>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            event: "?".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "?".to_string(),
            black: "?".to_string(),
            result: GameResult::Unknown,
            white_elo: None,
            black_elo: None,
            eco: None,
            event_date: None,
            extra_tags: Vec::new(),
            start_position: None,
            moves: Vec::new(),
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = Game::new();
        assert_eq!(game.event, "?");
        assert_eq!(game.result, GameResult::Unknown);
        assert_eq!(game.moves.len(), 0);
    }
}
