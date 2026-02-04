//! Full game decoding from SCID format

use super::{ByteBuffer, DecodeError, decode_move};
use crate::game::{Game, MoveNode};
use crate::position::Position;
use crate::types::{GameResult, Color, Square};

/// Flags for what to decode
pub const DECODE_NONE: u8 = 0;
pub const DECODE_TAGS: u8 = 1;
pub const DECODE_COMMENTS: u8 = 2;
pub const DECODE_ALL: u8 = 3;

/// Decode a complete game from SCID binary format
pub fn decode_game(buf: &mut ByteBuffer, flags: u8) -> Result<Game, DecodeError> {
    let mut game = Game::new();
    
    // Decode extra tags if requested
    if flags & DECODE_TAGS != 0 {
        decode_tags(buf, &mut game)?;
    } else {
        skip_tags(buf)?;
    }
    
    // Decode game flags
    let gflags = buf.get_byte()?;
    let non_standard_start = gflags & 1 != 0;
    let has_promotions = gflags & 2 != 0;
    let has_under_promotions = gflags & 4 != 0;
    
    // Decode starting position if non-standard
    if non_standard_start {
        let fen = buf.get_string()?;
        // TODO: Parse FEN and set game.start_position
        // For now, skip
    }
    
    // Decode the move tree
    let mut position = if non_standard_start {
        // TODO: Use FEN position
        Position::standard_start()
    } else {
        Position::standard_start()
    };
    
    game.moves = decode_variation(buf, &mut position, Color::White)?;
    
    // Decode comments if requested
    if flags & DECODE_COMMENTS != 0 {
        decode_comments(buf, &mut game)?;
    }
    
    Ok(game)
}

/// Decode extra PGN tags
fn decode_tags(buf: &mut ByteBuffer, game: &mut Game) -> Result<(), DecodeError> {
    // Read number of tags
    let num_tags = buf.get_byte()?;
    
    for _ in 0..num_tags {
        let tag_name = buf.get_string()?;
        let tag_value = buf.get_string()?;
        
        // Add to extra tags
        game.extra_tags.push((tag_name, tag_value));
    }
    
    Ok(())
}

/// Skip tags without decoding
fn skip_tags(buf: &mut ByteBuffer) -> Result<(), DecodeError> {
    let num_tags = buf.get_byte()?;
    
    for _ in 0..num_tags {
        buf.get_string()?; // tag name
        buf.get_string()?; // tag value
    }
    
    Ok(())
}

/// Decode a variation (sequence of moves)
fn decode_variation(
    buf: &mut ByteBuffer,
    position: &mut Position,
    mut current_color: Color,
) -> Result<Vec<MoveNode>, DecodeError> {
    let mut moves = Vec::new();
    
    // Get piece positions for move decoding
    // In a real implementation, we'd track this from the position
    let mut piece_positions = [Square::new(0, 0).unwrap(); 16];
    
    // Initialize with standard starting positions
    // This is simplified - real implementation would track position state
    piece_positions[0] = Square::new(4, 0).unwrap(); // White king on e1
    
    loop {
        // Check if we've hit a marker
        let byte = buf.peek_byte()?;
        
        // Special markers:
        // 0x00 with pieceNum 0 = null move
        // 0x?? with certain patterns = variation start/end
        
        if byte == 0xFF {
            // End of variation marker (simplified)
            buf.get_byte()?;
            break;
        }
        
        // Try to decode a move
        match decode_move(buf, &piece_positions, current_color) {
            Ok(move_data) => {
                // TODO: Generate SAN notation from move
                let san = format!("{}{}",
                    move_data.from.to_algebraic(),
                    move_data.to.to_algebraic()
                ); // Simplified - should generate proper SAN
                
                let move_node = MoveNode {
                    move_data,
                    san,
                    comment: None,
                    nags: Vec::new(),
                    variations: Vec::new(),
                };
                
                moves.push(move_node);
                
                // Switch colors
                current_color = current_color.opposite();
                
                // Update position (simplified)
                // Real implementation would update piece_positions
            }
            Err(e) => {
                // If we can't decode, we might have hit the end
                // Check for end marker
                if buf.remaining() == 0 {
                    break;
                }
                return Err(e);
            }
        }
        
        // Safety check to prevent infinite loops
        if moves.len() > 500 {
            break;
        }
    }
    
    Ok(moves)
}

/// Decode comments for moves
fn decode_comments(buf: &mut ByteBuffer, game: &mut Game) -> Result<(), DecodeError> {
    // Comments are stored after moves
    // Format: move_index (u16), comment_length (u16), comment_text
    
    while buf.remaining() > 0 {
        let move_index = buf.get_u16()? as usize;
        let comment_len = buf.get_u16()? as usize;
        
        if comment_len == 0 {
            break;
        }
        
        let mut comment_bytes = Vec::with_capacity(comment_len);
        for _ in 0..comment_len {
            comment_bytes.push(buf.get_byte()?);
        }
        
        let comment = String::from_utf8(comment_bytes)
            .map_err(|_| DecodeError::CorruptedData(buf.position()))?;
        
        // Attach comment to the move
        if move_index < game.moves.len() {
            game.moves[move_index].comment = Some(comment);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_tags() {
        let data = vec![0]; // 0 tags
        let mut buf = ByteBuffer::new(data);
        let mut game = Game::new();
        
        decode_tags(&mut buf, &mut game).unwrap();
        assert_eq!(game.extra_tags.len(), 0);
    }

    #[test]
    fn test_decode_single_tag() {
        let mut data = vec![1]; // 1 tag
        data.extend_from_slice(b"ECO\0");
        data.extend_from_slice(b"B12\0");
        
        let mut buf = ByteBuffer::new(data);
        let mut game = Game::new();
        
        decode_tags(&mut buf, &mut game).unwrap();
        assert_eq!(game.extra_tags.len(), 1);
        assert_eq!(game.extra_tags[0].0, "ECO");
        assert_eq!(game.extra_tags[0].1, "B12");
    }

    #[test]
    fn test_skip_tags() {
        let mut data = vec![2]; // 2 tags
        data.extend_from_slice(b"Tag1\0Value1\0");
        data.extend_from_slice(b"Tag2\0Value2\0");
        
        let mut buf = ByteBuffer::new(data);
        skip_tags(&mut buf).unwrap();
        
        // Should have consumed all tag data
        assert_eq!(buf.remaining(), 0);
    }
}
