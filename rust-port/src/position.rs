//! Position representation and manipulation

use crate::types::{Color, Square, Piece, PieceType};

/// Represents a chess position
#[derive(Debug, Clone)]
pub struct Position {
    // Board representation (0-63)
    board: [Option<Piece>; 64],
    
    // Game state
    to_move: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Position {
    pub fn new() -> Self {
        Position {
            board: [None; 64],
            to_move: Color::White,
            castling_rights: CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            },
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
    
    /// Set up the standard starting position
    pub fn standard_start() -> Self {
        let mut pos = Position::new();
        
        // Place pawns
        for file in 0..8 {
            pos.set_piece(Square::new(file, 1).unwrap(), 
                         Piece { piece_type: PieceType::Pawn, color: Color::White });
            pos.set_piece(Square::new(file, 6).unwrap(), 
                         Piece { piece_type: PieceType::Pawn, color: Color::Black });
        }
        
        // Place rooks
        pos.set_piece(Square::new(0, 0).unwrap(), 
                     Piece { piece_type: PieceType::Rook, color: Color::White });
        pos.set_piece(Square::new(7, 0).unwrap(), 
                     Piece { piece_type: PieceType::Rook, color: Color::White });
        pos.set_piece(Square::new(0, 7).unwrap(), 
                     Piece { piece_type: PieceType::Rook, color: Color::Black });
        pos.set_piece(Square::new(7, 7).unwrap(), 
                     Piece { piece_type: PieceType::Rook, color: Color::Black });
        
        // Place knights
        pos.set_piece(Square::new(1, 0).unwrap(), 
                     Piece { piece_type: PieceType::Knight, color: Color::White });
        pos.set_piece(Square::new(6, 0).unwrap(), 
                     Piece { piece_type: PieceType::Knight, color: Color::White });
        pos.set_piece(Square::new(1, 7).unwrap(), 
                     Piece { piece_type: PieceType::Knight, color: Color::Black });
        pos.set_piece(Square::new(6, 7).unwrap(), 
                     Piece { piece_type: PieceType::Knight, color: Color::Black });
        
        // Place bishops
        pos.set_piece(Square::new(2, 0).unwrap(), 
                     Piece { piece_type: PieceType::Bishop, color: Color::White });
        pos.set_piece(Square::new(5, 0).unwrap(), 
                     Piece { piece_type: PieceType::Bishop, color: Color::White });
        pos.set_piece(Square::new(2, 7).unwrap(), 
                     Piece { piece_type: PieceType::Bishop, color: Color::Black });
        pos.set_piece(Square::new(5, 7).unwrap(), 
                     Piece { piece_type: PieceType::Bishop, color: Color::Black });
        
        // Place queens
        pos.set_piece(Square::new(3, 0).unwrap(), 
                     Piece { piece_type: PieceType::Queen, color: Color::White });
        pos.set_piece(Square::new(3, 7).unwrap(), 
                     Piece { piece_type: PieceType::Queen, color: Color::Black });
        
        // Place kings
        pos.set_piece(Square::new(4, 0).unwrap(), 
                     Piece { piece_type: PieceType::King, color: Color::White });
        pos.set_piece(Square::new(4, 7).unwrap(), 
                     Piece { piece_type: PieceType::King, color: Color::Black });
        
        pos
    }
    
    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.board[square.0 as usize] = Some(piece);
    }
    
    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.board[square.0 as usize]
    }
    
    pub fn to_move(&self) -> Color {
        self.to_move
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_position() {
        let pos = Position::new();
        assert_eq!(pos.to_move(), Color::White);
        assert_eq!(pos.fullmove_number, 1);
    }

    #[test]
    fn test_standard_start() {
        let pos = Position::standard_start();
        
        // Check white pawns
        for file in 0..8 {
            let sq = Square::new(file, 1).unwrap();
            let piece = pos.get_piece(sq).unwrap();
            assert_eq!(piece.piece_type, PieceType::Pawn);
            assert_eq!(piece.color, Color::White);
        }
        
        // Check black pawns
        for file in 0..8 {
            let sq = Square::new(file, 6).unwrap();
            let piece = pos.get_piece(sq).unwrap();
            assert_eq!(piece.piece_type, PieceType::Pawn);
            assert_eq!(piece.color, Color::Black);
        }
        
        // Check white king
        let white_king_sq = Square::new(4, 0).unwrap();
        let white_king = pos.get_piece(white_king_sq).unwrap();
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.color, Color::White);
    }
}
