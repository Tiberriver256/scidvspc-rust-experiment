//! Core types for chess representation

use std::fmt;

/// Represents a square on the chess board (0-63)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub(crate) u8);

impl Square {
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Square(rank * 8 + file))
        } else {
            None
        }
    }
    
    pub fn from_index(index: u8) -> Option<Self> {
        if index < 64 {
            Some(Square(index))
        } else {
            None
        }
    }
    
    pub fn file(self) -> u8 {
        self.0 % 8
    }
    
    pub fn rank(self) -> u8 {
        self.0 / 8
    }
    
    pub fn to_algebraic(self) -> String {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        format!("{}{}", file, rank)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

/// Chess piece types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Chess colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// A chess piece (type + color)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

/// Represents a chess move
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub captured: Option<Piece>,
    pub promotion: Option<PieceType>,
    pub is_en_passant: bool,
    pub is_castling: bool,
}

/// Game result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    White,
    Black,
    Draw,
    Unknown,
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameResult::White => write!(f, "1-0"),
            GameResult::Black => write!(f, "0-1"),
            GameResult::Draw => write!(f, "1/2-1/2"),
            GameResult::Unknown => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_creation() {
        let sq = Square::new(0, 0).unwrap(); // a1
        assert_eq!(sq.file(), 0);
        assert_eq!(sq.rank(), 0);
        assert_eq!(sq.to_algebraic(), "a1");
    }

    #[test]
    fn test_square_e4() {
        let sq = Square::new(4, 3).unwrap(); // e4
        assert_eq!(sq.to_algebraic(), "e4");
    }

    #[test]
    fn test_square_h8() {
        let sq = Square::new(7, 7).unwrap(); // h8
        assert_eq!(sq.to_algebraic(), "h8");
    }

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn test_game_result_display() {
        assert_eq!(format!("{}", GameResult::White), "1-0");
        assert_eq!(format!("{}", GameResult::Black), "0-1");
        assert_eq!(format!("{}", GameResult::Draw), "1/2-1/2");
        assert_eq!(format!("{}", GameResult::Unknown), "*");
    }
}
