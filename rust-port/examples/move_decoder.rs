// SCID binary move decoder using shakmaty
// Decodes moves from SCID's compressed binary format

use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square};
use shakmaty::san::SanPlus;

pub struct MoveDecoder {
    chess: Chess,
}

impl MoveDecoder {
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let setup: shakmaty::fen::Fen = fen.parse()
            .map_err(|e| format!("Invalid FEN: {:?}", e))?;
        
        let chess: Chess = setup.into_position(shakmaty::CastlingMode::Standard)
            .map_err(|e| format!("Cannot create position: {:?}", e))?;
        
        Ok(MoveDecoder { chess })
    }
    
    pub fn apply_move(&mut self, mv: &Move) -> Result<(), String> {
        self.chess = self.chess.clone().play(mv)
            .map_err(|e| format!("Invalid move: {:?}", e))?;
        Ok(())
    }
    
    pub fn get_board(&self) -> &shakmaty::Board {
        self.chess.board()
    }
    
    pub fn get_piece_list(&self) -> Vec<Square> {
        let mut pieces = Vec::new();
        let color = self.chess.turn();
        
        // SCID order: pieces in FEN order, but when King is encountered:
        // - King goes to position 0
        // - Piece that was at position 0 moves to the end
        // - All other pieces stay in their positions
        
        // Collect pieces in FEN order (rank 8→1, file a→h)
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::from_coords(
                    shakmaty::File::try_from(file).unwrap(),
                    shakmaty::Rank::try_from(rank).unwrap()
                );
                if let Some(piece) = self.chess.board().piece_at(sq) {
                    if piece.color == color {
                        if piece.role == Role::King {
                            // King goes to position 0
                            // Move current piece at [0] (if any) to the end
                            if !pieces.is_empty() {
                                let first = pieces[0];
                                pieces.remove(0);
                                pieces.insert(0, sq);  // King at front
                                pieces.push(first);     // Old [0] at end
                            } else {
                                pieces.push(sq);
                            }
                        } else {
                            // Non-King: add to end
                            pieces.push(sq);
                        }
                    }
                }
            }
        }
        
        pieces
    }
    
    pub fn decode_move(&self, byte_val: u8, next_byte: Option<u8>) -> Result<Move, String> {
        let piece_num = byte_val >> 4;
        let val = byte_val & 15;
        
        let piece_list = self.get_piece_list();
        if piece_num as usize >= piece_list.len() {
            return Err(format!("Invalid piece_num: {} (list size: {})", piece_num, piece_list.len()));
        }
        
        let from = piece_list[piece_num as usize];
        let piece = self.chess.board().piece_at(from)
            .ok_or_else(|| format!("No piece at {}", from))?;
        
        let to = match piece.role {
            Role::King => self.decode_king(from, val)?,
            Role::Queen => self.decode_queen(from, val, next_byte)?,
            Role::Rook => self.decode_rook(from, val)?,
            Role::Bishop => self.decode_bishop(from, val)?,
            Role::Knight => self.decode_knight(from, val)?,
            Role::Pawn => {
                let (to, promote) = self.decode_pawn(from, val)?;
                let capture = self.chess.board().piece_at(to).map(|p| p.role);
                return Ok(Move::Normal {
                    role: Role::Pawn,
                    from,
                    to,
                    capture,
                    promotion: promote,
                });
            }
        };
        
        // Check for castling
        if piece.role == Role::King {
            let from_file = from.file();
            let to_file = to.file();
            
            if from_file == shakmaty::File::E {
                if to_file == shakmaty::File::G {
                    // Kingside castling
                    let rook_sq = Square::from_coords(shakmaty::File::H, from.rank());
                    return Ok(Move::Castle { king: from, rook: rook_sq });
                } else if to_file == shakmaty::File::C {
                    // Queenside castling
                    let rook_sq = Square::from_coords(shakmaty::File::A, from.rank());
                    return Ok(Move::Castle { king: from, rook: rook_sq });
                }
            }
        }
        
        let capture = self.chess.board().piece_at(to).map(|p| p.role);
        
        Ok(Move::Normal {
            role: piece.role,
            from,
            to,
            capture,
            promotion: None,
        })
    }
    
    fn decode_king(&self, from: Square, val: u8) -> Result<Square, String> {
        const SQ_DIFF: [i8; 11] = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];
        
        if val == 0 {
            return Ok(from); // Null move
        }
        
        if val < 1 || val > 10 {
            return Err(format!("Invalid king move value: {}", val));
        }
        
        let from_idx = from as u8;
        let new_idx = from_idx as i8 + SQ_DIFF[val as usize];
        
        if new_idx < 0 || new_idx >= 64 {
            return Err(format!("King move out of bounds: {}", new_idx));
        }
        
        Square::try_from(new_idx as u8)
            .map_err(|_| format!("Invalid square: {}", new_idx))
    }
    
    fn decode_knight(&self, from: Square, val: u8) -> Result<Square, String> {
        const SQ_DIFF: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];
        
        if val < 1 || val > 8 {
            return Err(format!("Invalid knight move value: {}", val));
        }
        
        let from_idx = from as u8;
        let new_idx = from_idx as i8 + SQ_DIFF[val as usize];
        
        if new_idx < 0 || new_idx >= 64 {
            return Err(format!("Knight move out of bounds: {}", new_idx));
        }
        
        Square::try_from(new_idx as u8)
            .map_err(|_| format!("Invalid square: {}", new_idx))
    }
    
    fn decode_rook(&self, from: Square, val: u8) -> Result<Square, String> {
        if val >= 8 {
            // Move along file to different rank
            let new_rank = shakmaty::Rank::try_from(val - 8)
                .map_err(|_| format!("Invalid rank: {}", val - 8))?;
            Ok(Square::from_coords(from.file(), new_rank))
        } else {
            // Move along rank to different file
            let new_file = shakmaty::File::try_from(val)
                .map_err(|_| format!("Invalid file: {}", val))?;
            Ok(Square::from_coords(new_file, from.rank()))
        }
    }
    
    fn decode_bishop(&self, from: Square, val: u8) -> Result<Square, String> {
        let file = val & 7;
        let file_diff = file as i8 - from.file() as i8;
        
        let from_idx = from as u8;
        let new_idx = if val >= 8 {
            // Up-left/down-right direction
            from_idx as i8 - 7 * file_diff
        } else {
            // Up-right/down-left direction
            from_idx as i8 + 9 * file_diff
        };
        
        if new_idx < 0 || new_idx >= 64 {
            return Err(format!("Bishop move out of bounds: {}", new_idx));
        }
        
        Square::try_from(new_idx as u8)
            .map_err(|_| format!("Invalid square: {}", new_idx))
    }
    
    fn decode_queen(&self, from: Square, val: u8, next_byte: Option<u8>) -> Result<Square, String> {
        if val >= 8 {
            // Rook-vertical move
            let new_rank = shakmaty::Rank::try_from(val - 8)
                .map_err(|_| format!("Invalid rank: {}", val - 8))?;
            Ok(Square::from_coords(from.file(), new_rank))
        } else if val as u8 != from.file() as u8 {
            // Rook-horizontal move
            let new_file = shakmaty::File::try_from(val)
                .map_err(|_| format!("Invalid file: {}", val))?;
            Ok(Square::from_coords(new_file, from.rank()))
        } else {
            // Diagonal move - needs second byte
            let next = next_byte.ok_or("Queen diagonal move needs second byte")?;
            if next < 64 || next > 127 {
                return Err(format!("Invalid second byte for queen diagonal: {}", next));
            }
            Square::try_from(next - 64)
                .map_err(|_| format!("Invalid square: {}", next - 64))
        }
    }
    
    fn decode_pawn(&self, from: Square, val: u8) -> Result<(Square, Option<Role>), String> {
        const TO_SQ_DIFF: [u8; 16] = [
            7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 16
        ];
        
        const PROMO_ROLE: [Option<Role>; 16] = [
            None, None, None,
            Some(Role::Queen), Some(Role::Queen), Some(Role::Queen),
            Some(Role::Rook), Some(Role::Rook), Some(Role::Rook),
            Some(Role::Bishop), Some(Role::Bishop), Some(Role::Bishop),
            Some(Role::Knight), Some(Role::Knight), Some(Role::Knight),
            None,
        ];
        
        if val >= 16 {
            return Err(format!("Invalid pawn move value: {}", val));
        }
        
        let diff = TO_SQ_DIFF[val as usize] as i8;
        let from_idx = from as u8;
        let new_idx = match self.chess.turn() {
            Color::White => from_idx as i8 + diff,
            Color::Black => from_idx as i8 - diff,
        };
        
        if new_idx < 0 || new_idx >= 64 {
            return Err(format!("Pawn move out of bounds: {}", new_idx));
        }
        
        let to = Square::try_from(new_idx as u8)
            .map_err(|_| format!("Invalid square: {}", new_idx))?;
        
        Ok((to, PROMO_ROLE[val as usize]))
    }
    
    pub fn move_to_san(&self, mv: &Move) -> String {
        let san_plus = SanPlus::from_move(self.chess.clone(), mv);
        san_plus.to_string()
    }
}
