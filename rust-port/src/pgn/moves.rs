//! Move list formatting for PGN output

use crate::game::{Game, MoveNode};
use crate::types::{Color, PieceType};
use super::{PgnFormat, PgnOptions};

pub struct MoveFormatter {
    options: PgnOptions,
}

impl MoveFormatter {
    pub fn new(options: PgnOptions) -> Self {
        MoveFormatter { options }
    }
    
    /// Format a complete move list as PGN
    pub fn format_moves(&self, game: &Game) -> String {
        let mut output = String::new();
        let mut move_number = 1;
        let mut last_color = Color::Black; // Start assuming we need move number
        
        for (i, move_node) in game.moves.iter().enumerate() {
            // Determine if we need to print move number
            let current_color = if i % 2 == 0 { Color::White } else { Color::Black };
            
            if current_color == Color::White {
                // White move: always show number
                output.push_str(&format!("{}. ", move_number));
            } else if last_color == Color::Black || i == 0 {
                // Black move that doesn't follow white (e.g., after variation)
                output.push_str(&format!("{}... ", move_number));
            }
            
            // Add the move in SAN
            output.push_str(&move_node.san);
            
            // Add NAGs if present and enabled
            if self.options.include_nags && !move_node.nags.is_empty() {
                for &nag in &move_node.nags {
                    output.push(' ');
                    output.push_str(&self.format_nag(nag));
                }
            }
            
            // Add comment if present and enabled
            if self.options.include_comments {
                if let Some(ref comment) = move_node.comment {
                    output.push_str(" {");
                    output.push_str(comment);
                    output.push('}');
                }
            }
            
            // Add variations if present and enabled
            if self.options.include_variations && !move_node.variations.is_empty() {
                for variation in &move_node.variations {
                    output.push_str(" (");
                    output.push_str(&self.format_variation(variation, move_number, current_color));
                    output.push(')');
                }
            }
            
            output.push(' ');
            
            // Update for next iteration
            if current_color == Color::Black {
                move_number += 1;
            }
            last_color = current_color;
        }
        
        output.trim_end().to_string()
    }
    
    fn format_variation(&self, moves: &[MoveNode], start_number: usize, start_color: Color) -> String {
        let mut output = String::new();
        let mut move_number = start_number;
        let mut current_color = start_color;
        
        for (i, move_node) in moves.iter().enumerate() {
            if i == 0 && current_color == Color::White {
                output.push_str(&format!("{}. ", move_number));
            } else if i == 0 && current_color == Color::Black {
                output.push_str(&format!("{}... ", move_number));
            } else if current_color == Color::White {
                output.push_str(&format!("{}. ", move_number));
            }
            
            output.push_str(&move_node.san);
            
            if self.options.include_comments {
                if let Some(ref comment) = move_node.comment {
                    output.push_str(" {");
                    output.push_str(comment);
                    output.push('}');
                }
            }
            
            output.push(' ');
            
            if current_color == Color::Black {
                move_number += 1;
                current_color = Color::White;
            } else {
                current_color = Color::Black;
            }
        }
        
        output.trim_end().to_string()
    }
    
    fn format_nag(&self, nag: u8) -> String {
        match self.options.format {
            PgnFormat::Plain | PgnFormat::Html | PgnFormat::Color => {
                // Use symbolic NAGs for common ones
                match nag {
                    1 => "!".to_string(),      // Good move
                    2 => "?".to_string(),      // Poor move
                    3 => "!!".to_string(),     // Excellent move
                    4 => "??".to_string(),     // Blunder
                    5 => "!?".to_string(),     // Interesting move
                    6 => "?!".to_string(),     // Dubious move
                    10 => "=".to_string(),     // Equal position
                    13 => "∞".to_string(),     // Unclear
                    14 => "⩲".to_string(),     // White is slightly better
                    15 => "⩱".to_string(),     // Black is slightly better
                    16 => "±".to_string(),     // White is better
                    17 => "∓".to_string(),     // Black is better
                    18 => "+-".to_string(),    // White is winning
                    19 => "-+".to_string(),    // Black is winning
                    _ => format!("${}", nag),  // Numeric format for others
                }
            }
            PgnFormat::Latex => {
                format!("$${}", nag)
            }
        }
    }
}

/// Generate SAN notation for a move
/// This is a simplified version - full implementation would need position analysis
pub fn generate_san(
    piece_type: PieceType,
    from_square: &str,
    to_square: &str,
    is_capture: bool,
    is_promotion: Option<PieceType>,
    is_check: bool,
    is_mate: bool,
) -> String {
    let mut san = String::new();
    
    match piece_type {
        PieceType::King => {
            // Check for castling
            if from_square == "e1" && to_square == "g1" {
                return "O-O".to_string(); // White kingside
            } else if from_square == "e1" && to_square == "c1" {
                return "O-O-O".to_string(); // White queenside
            } else if from_square == "e8" && to_square == "g8" {
                return "O-O".to_string(); // Black kingside
            } else if from_square == "e8" && to_square == "c8" {
                return "O-O-O".to_string(); // Black queenside
            }
            san.push('K');
        }
        PieceType::Queen => san.push('Q'),
        PieceType::Rook => san.push('R'),
        PieceType::Bishop => san.push('B'),
        PieceType::Knight => san.push('N'),
        PieceType::Pawn => {
            // For pawns, only show file if capture
            if is_capture {
                san.push(from_square.chars().next().unwrap());
            }
        }
    }
    
    if is_capture {
        san.push('x');
    }
    
    san.push_str(to_square);
    
    // Promotion
    if let Some(promo) = is_promotion {
        san.push('=');
        match promo {
            PieceType::Queen => san.push('Q'),
            PieceType::Rook => san.push('R'),
            PieceType::Bishop => san.push('B'),
            PieceType::Knight => san.push('N'),
            _ => {}
        }
    }
    
    // Check/mate
    if is_mate {
        san.push('#');
    } else if is_check {
        san.push('+');
    }
    
    san
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;
    use crate::types::{GameResult, Move, Piece, Square, Color};

    #[test]
    fn test_simple_move_formatting() {
        let san = generate_san(
            PieceType::Pawn,
            "e2",
            "e4",
            false,
            None,
            false,
            false,
        );
        assert_eq!(san, "e4");
    }

    #[test]
    fn test_capture_formatting() {
        let san = generate_san(
            PieceType::Pawn,
            "e4",
            "d5",
            true,
            None,
            false,
            false,
        );
        assert_eq!(san, "exd5");
    }

    #[test]
    fn test_kingside_castle() {
        let san = generate_san(
            PieceType::King,
            "e1",
            "g1",
            false,
            None,
            false,
            false,
        );
        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_queenside_castle() {
        let san = generate_san(
            PieceType::King,
            "e1",
            "c1",
            false,
            None,
            false,
            false,
        );
        assert_eq!(san, "O-O-O");
    }

    #[test]
    fn test_knight_move() {
        let san = generate_san(
            PieceType::Knight,
            "g1",
            "f3",
            false,
            None,
            false,
            false,
        );
        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_check() {
        let san = generate_san(
            PieceType::Queen,
            "d1",
            "h5",
            false,
            None,
            true,
            false,
        );
        assert_eq!(san, "Qh5+");
    }

    #[test]
    fn test_mate() {
        let san = generate_san(
            PieceType::Queen,
            "h5",
            "f7",
            true,
            None,
            false,
            true,
        );
        assert_eq!(san, "Qxf7#");
    }

    #[test]
    fn test_promotion() {
        let san = generate_san(
            PieceType::Pawn,
            "e7",
            "e8",
            false,
            Some(PieceType::Queen),
            false,
            false,
        );
        assert_eq!(san, "e8=Q");
    }

    #[test]
    fn test_nag_formatting() {
        let formatter = MoveFormatter::new(PgnOptions::default());
        
        assert_eq!(formatter.format_nag(1), "!");
        assert_eq!(formatter.format_nag(2), "?");
        assert_eq!(formatter.format_nag(3), "!!");
        assert_eq!(formatter.format_nag(4), "??");
        assert_eq!(formatter.format_nag(5), "!?");
        assert_eq!(formatter.format_nag(6), "?!");
    }
}
