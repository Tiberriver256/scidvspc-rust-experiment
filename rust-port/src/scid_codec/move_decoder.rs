//! SCID binary format decoder
//! 
//! This module decodes games from SCID's compact binary format.
//! The encoding is documented in game.cpp functions like DecodeNextMove().

use crate::types::{Square, PieceType, Color, Piece, Move};
use crate::game::{Game, MoveNode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Unexpected end of data")]
    UnexpectedEof,
    
    #[error("Invalid move encoding: {0}")]
    InvalidMove(String),
    
    #[error("Invalid piece number: {0}")]
    InvalidPiece(u8),
    
    #[error("Invalid square: {0}")]
    InvalidSquare(u8),
    
    #[error("Corrupted data at offset {0}")]
    CorruptedData(usize),
}

/// A simple byte buffer reader
pub struct ByteBuffer {
    data: Vec<u8>,
    pos: usize,
}

impl ByteBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        ByteBuffer { data, pos: 0 }
    }
    
    pub fn get_byte(&mut self) -> Result<u8, DecodeError> {
        if self.pos >= self.data.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let byte = self.data[self.pos];
        self.pos += 1;
        Ok(byte)
    }
    
    pub fn peek_byte(&self) -> Result<u8, DecodeError> {
        if self.pos >= self.data.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        Ok(self.data[self.pos])
    }
    
    pub fn get_u16(&mut self) -> Result<u16, DecodeError> {
        let b1 = self.get_byte()? as u16;
        let b2 = self.get_byte()? as u16;
        Ok((b1 << 8) | b2)
    }
    
    pub fn get_u32(&mut self) -> Result<u32, DecodeError> {
        let b1 = self.get_byte()? as u32;
        let b2 = self.get_byte()? as u32;
        let b3 = self.get_byte()? as u32;
        let b4 = self.get_byte()? as u32;
        Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
    }
    
    /// Read a null-terminated string
    pub fn get_string(&mut self) -> Result<String, DecodeError> {
        let mut bytes = Vec::new();
        loop {
            let byte = self.get_byte()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        String::from_utf8(bytes)
            .map_err(|_| DecodeError::CorruptedData(self.pos))
    }
    
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
    
    pub fn position(&self) -> usize {
        self.pos
    }
}

/// SCID move encoding helpers
/// Moves are encoded as (pieceNum << 4) | value
fn decode_move_byte(byte: u8) -> (u8, u8) {
    let piece_num = (byte >> 4) & 0x0F;
    let value = byte & 0x0F;
    (piece_num, value)
}

/// Decode a King move
/// King moves are encoded as differences from the current square
fn decode_king_move(value: u8, from_square: Square) -> Result<Square, DecodeError> {
    // Square differences for king moves
    // val: 0=null, 1=-9, 2=-8, 3=-7, 4=-1, 5=+1, 6=+7, 7=+8, 8=+9, 9=-2 (Q-castle), 10=+2 (K-castle)
    const SQ_DIFF: [i8; 11] = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];
    
    if value == 0 {
        // Null move
        return Ok(from_square);
    }
    
    if value > 10 {
        return Err(DecodeError::InvalidMove(format!("Invalid king move value: {}", value)));
    }
    
    let from_idx = from_square.0 as i8;
    let to_idx = from_idx + SQ_DIFF[value as usize];
    
    if to_idx < 0 || to_idx > 63 {
        return Err(DecodeError::InvalidSquare(to_idx as u8));
    }
    
    Square::from_index(to_idx as u8)
        .ok_or(DecodeError::InvalidSquare(to_idx as u8))
}

/// Decode a Knight move
fn decode_knight_move(value: u8, from_square: Square) -> Result<Square, DecodeError> {
    // val: 1=-17, 2=-15, 3=-10, 4=-6, 5=+6, 6=+10, 7=+15, 8=+17
    const SQ_DIFF: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];
    
    if value < 1 || value > 8 {
        return Err(DecodeError::InvalidMove(format!("Invalid knight move value: {}", value)));
    }
    
    let from_idx = from_square.0 as i8;
    let to_idx = from_idx + SQ_DIFF[value as usize];
    
    if to_idx < 0 || to_idx > 63 {
        return Err(DecodeError::InvalidSquare(to_idx as u8));
    }
    
    Square::from_index(to_idx as u8)
        .ok_or(DecodeError::InvalidSquare(to_idx as u8))
}

/// Decode a Rook move
fn decode_rook_move(value: u8, from_square: Square) -> Result<Square, DecodeError> {
    // Values 0-7: same rank, different file
    // Values 8-15: same file, different rank
    if value >= 8 {
        // Same file, different rank
        let rank = value - 8;
        Square::new(from_square.file(), rank)
    } else {
        // Same rank, different file
        Square::new(value, from_square.rank())
    }
    .ok_or(DecodeError::InvalidMove(format!("Invalid rook move value: {}", value)))
}

/// Decode a Bishop move
fn decode_bishop_move(value: u8, from_square: Square) -> Result<Square, DecodeError> {
    let file = value & 7;
    let file_diff = file as i8 - from_square.file() as i8;
    
    let to_idx = if value >= 8 {
        // Up-left or down-right
        from_square.0 as i8 - 7 * file_diff
    } else {
        // Up-right or down-left
        from_square.0 as i8 + 9 * file_diff
    };
    
    if to_idx < 0 || to_idx > 63 {
        return Err(DecodeError::InvalidSquare(to_idx as u8));
    }
    
    Square::from_index(to_idx as u8)
        .ok_or(DecodeError::InvalidSquare(to_idx as u8))
}

/// Decode a Queen move
/// Queens can move like rooks or bishops, with special encoding for diagonal moves
fn decode_queen_move(value: u8, from_square: Square, buf: &mut ByteBuffer) -> Result<Square, DecodeError> {
    if value == from_square.file() {
        // This is a diagonal move (indicated by horizontal move to same square)
        // Next byte contains the target square + 64
        let next_byte = buf.get_byte()?;
        if next_byte < 64 {
            return Err(DecodeError::InvalidMove("Queen diagonal move marker too small".to_string()));
        }
        let to_square = next_byte - 64;
        Square::from_index(to_square)
            .ok_or(DecodeError::InvalidSquare(to_square))
    } else {
        // Rook-like move
        decode_rook_move(value, from_square)
    }
}

/// Decode a Pawn move
/// Pawns have special encoding for captures and promotions
fn decode_pawn_move(value: u8, from_square: Square, color: Color) -> Result<(Square, Option<PieceType>), DecodeError> {
    // For pawns: 
    // 0-7: non-capture moves forward
    // 8-15: captures (with promotion info if on 8th/1st rank)
    
    let is_capture = value >= 8;
    let base_value = if is_capture { value - 8 } else { value };
    
    let direction = if color == Color::White { 8 } else { -8 };
    let to_square_idx = from_square.0 as i8 + direction;
    
    // TODO: Handle pawn captures and promotions properly
    // This is simplified - real implementation needs position context
    
    if to_square_idx < 0 || to_square_idx > 63 {
        return Err(DecodeError::InvalidSquare(to_square_idx as u8));
    }
    
    let to_square = Square::from_index(to_square_idx as u8)
        .ok_or(DecodeError::InvalidSquare(to_square_idx as u8))?;
    
    // Check for promotion
    let promotion = if to_square.rank() == 0 || to_square.rank() == 7 {
        // Promotion - decode which piece
        Some(PieceType::Queen) // Simplified
    } else {
        None
    };
    
    Ok((to_square, promotion))
}

/// Main decoder for a single move
pub fn decode_move(
    buf: &mut ByteBuffer,
    piece_positions: &[Square; 16],  // Positions of all pieces for current color
    current_color: Color,
) -> Result<Move, DecodeError> {
    let move_byte = buf.get_byte()?;
    let (piece_num, value) = decode_move_byte(move_byte);
    
    if piece_num >= 16 {
        return Err(DecodeError::InvalidPiece(piece_num));
    }
    
    let from_square = piece_positions[piece_num as usize];
    
    // Determine piece type from piece number
    // In SCID: 0=King, 1-8 = pawns, 9-10 = knights, 11-12 = bishops, 13-14 = rooks, 15 = queen
    let (piece_type, to_square, promotion) = if piece_num == 0 {
        // King
        (PieceType::King, decode_king_move(value, from_square)?, None)
    } else if piece_num <= 8 {
        // Pawn
        let (to_sq, promo) = decode_pawn_move(value, from_square, current_color)?;
        (PieceType::Pawn, to_sq, promo)
    } else if piece_num <= 10 {
        // Knight
        (PieceType::Knight, decode_knight_move(value, from_square)?, None)
    } else if piece_num <= 12 {
        // Bishop
        (PieceType::Bishop, decode_bishop_move(value, from_square)?, None)
    } else if piece_num <= 14 {
        // Rook
        (PieceType::Rook, decode_rook_move(value, from_square)?, None)
    } else {
        // Queen
        (PieceType::Queen, decode_queen_move(value, from_square, buf)?, None)
    };
    
    // TODO: Determine if capture by checking board state
    let captured = None;
    
    Ok(Move {
        from: from_square,
        to: to_square,
        piece: Piece { piece_type, color: current_color },
        captured,
        promotion,
        is_en_passant: false,
        is_castling: piece_type == PieceType::King && 
                     ((from_square.0 as i8 - to_square.0 as i8).abs() == 2),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_buffer() {
        let mut buf = ByteBuffer::new(vec![0x12, 0x34, 0x56, 0x78]);
        assert_eq!(buf.get_byte().unwrap(), 0x12);
        assert_eq!(buf.get_byte().unwrap(), 0x34);
        assert_eq!(buf.remaining(), 2);
    }

    #[test]
    fn test_decode_move_byte() {
        let byte = 0x35; // piece 3, value 5
        let (piece, value) = decode_move_byte(byte);
        assert_eq!(piece, 3);
        assert_eq!(value, 5);
    }

    #[test]
    fn test_decode_king_move() {
        let from = Square::new(4, 0).unwrap(); // e1
        
        // Kingside castling (value 10, +2 squares)
        let to = decode_king_move(10, from).unwrap();
        assert_eq!(to, Square::new(6, 0).unwrap()); // g1
        
        // Queenside castling (value 9, -2 squares)
        let to = decode_king_move(9, from).unwrap();
        assert_eq!(to, Square::new(2, 0).unwrap()); // c1
    }

    #[test]
    fn test_decode_knight_move() {
        let from = Square::new(6, 0).unwrap(); // g1
        
        // Knight to f3 (difference -17 + 16 = -1 in encoding)
        // Actually knight from g1 to f3 is g1(6) -> f3(21), diff = 15
        // So this would be value 7
        let to = decode_knight_move(7, from).unwrap();
        assert_eq!(to, Square::new(5, 2).unwrap()); // f3
    }

    #[test]
    fn test_string_reading() {
        let mut buf = ByteBuffer::new(vec![b'H', b'e', b'l', b'l', b'o', 0, b'W']);
        let s = buf.get_string().unwrap();
        assert_eq!(s, "Hello");
        assert_eq!(buf.get_byte().unwrap(), b'W');
    }
}
