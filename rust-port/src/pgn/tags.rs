//! PGN tag handling

use crate::game::Game;
use super::PgnFormat;

pub struct TagWriter;

impl TagWriter {
    pub fn write_tags(game: &Game, format: PgnFormat, short_header: bool) -> String {
        if short_header {
            Self::write_short_header(game, format)
        } else {
            Self::write_standard_tags(game, format)
        }
    }
    
    fn write_standard_tags(game: &Game, _format: PgnFormat) -> String {
        let mut output = String::new();
        
        // Standard seven tags
        output.push_str(&format!("[Event \"{}\"]\n", game.event));
        output.push_str(&format!("[Site \"{}\"]\n", game.site));
        output.push_str(&format!("[Date \"{}\"]\n", game.date));
        output.push_str(&format!("[Round \"{}\"]\n", game.round));
        output.push_str(&format!("[White \"{}\"]\n", game.white));
        output.push_str(&format!("[Black \"{}\"]\n", game.black));
        output.push_str(&format!("[Result \"{}\"]\n", game.result));
        
        // Optional tags
        if let Some(white_elo) = game.white_elo {
            output.push_str(&format!("[WhiteElo \"{}\"]\n", white_elo));
        }
        
        if let Some(black_elo) = game.black_elo {
            output.push_str(&format!("[BlackElo \"{}\"]\n", black_elo));
        }
        
        if let Some(ref eco) = game.eco {
            output.push_str(&format!("[ECO \"{}\"]\n", eco));
        }
        
        if let Some(ref event_date) = game.event_date {
            output.push_str(&format!("[EventDate \"{}\"]\n", event_date));
        }
        
        // Extra tags
        for (tag, value) in &game.extra_tags {
            output.push_str(&format!("[{} \"{}\"]\n", tag, value));
        }
        
        output
    }
    
    fn write_short_header(game: &Game, _format: PgnFormat) -> String {
        let mut output = String::new();
        
        // First line: White - Black with Elo ratings
        output.push_str(&game.white);
        if let Some(elo) = game.white_elo {
            output.push_str(&format!("  ({})", elo));
        }
        output.push_str("   --   ");
        output.push_str(&game.black);
        if let Some(elo) = game.black_elo {
            output.push_str(&format!("  ({})", elo));
        }
        output.push('\n');
        
        // Second line: Event, Site, Date, Result
        output.push_str(&game.event);
        if game.round != "?" && !game.round.is_empty() {
            output.push_str(&format!("  (Round {})", game.round));
        }
        output.push_str("  ");
        if game.site != "?" && !game.site.is_empty() {
            output.push_str(&game.site);
            output.push_str("  ");
        }
        output.push_str(&game.date);
        output.push_str("  ");
        output.push_str(&format!("{}", game.result));
        if let Some(ref eco) = game.eco {
            output.push_str(&format!("  {}", eco));
        }
        output.push('\n');
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GameResult;

    #[test]
    fn test_standard_tags() {
        let mut game = Game::new();
        game.event = "World Championship".to_string();
        game.site = "Moscow".to_string();
        game.date = "1985.09.03".to_string();
        game.round = "1".to_string();
        game.white = "Kasparov, Garry".to_string();
        game.black = "Karpov, Anatoly".to_string();
        game.result = GameResult::White;
        game.white_elo = Some(2700);
        game.black_elo = Some(2720);
        
        let tags = TagWriter::write_tags(&game, PgnFormat::Plain, false);
        
        assert!(tags.contains("[Event \"World Championship\"]"));
        assert!(tags.contains("[White \"Kasparov, Garry\"]"));
        assert!(tags.contains("[WhiteElo \"2700\"]"));
        assert!(tags.contains("[Result \"1-0\"]"));
    }

    #[test]
    fn test_short_header() {
        let mut game = Game::new();
        game.white = "Kasparov, Garry".to_string();
        game.black = "Karpov, Anatoly".to_string();
        game.white_elo = Some(2700);
        game.black_elo = Some(2720);
        
        let tags = TagWriter::write_tags(&game, PgnFormat::Plain, true);
        
        assert!(tags.contains("Kasparov, Garry  (2700)"));
        assert!(tags.contains("Karpov, Anatoly  (2720)"));
    }
}
