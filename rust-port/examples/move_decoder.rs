// SCID binary move decoder using shakmaty
// Decodes moves from SCID's compressed binary format

use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square, EnPassantMode, CastlingMode};
use shakmaty::san::SanPlus;

#[derive(Clone)]
pub struct MoveDecoder {
    chess: Chess,
    white_pieces: Vec<Square>,  // Maintained piece list for white
    black_pieces: Vec<Square>,  // Maintained piece list for black
}

impl MoveDecoder {
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let setup: shakmaty::fen::Fen = fen.parse()
            .map_err(|e| format!("Invalid FEN: {:?}", e))?;
        
        let chess: Chess = setup.into_position(shakmaty::CastlingMode::Standard)
            .map_err(|e| format!("Cannot create position: {:?}", e))?;
        
        // Build initial piece lists
        let white_pieces = Self::build_piece_list(&chess, Color::White);
        let black_pieces = Self::build_piece_list(&chess, Color::Black);
        
        Ok(MoveDecoder { chess, white_pieces, black_pieces })
    }
    
    fn build_piece_list(chess: &Chess, color: Color) -> Vec<Square> {
        let mut pieces = Vec::new();
        
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
                if let Some(piece) = chess.board().piece_at(sq) {
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
    
    pub fn apply_move(&mut self, mv: &Move) -> Result<(), String> {
        // Update piece lists to match C++'s: List[ToMove][sm->pieceNum] = to
        // The piece list must be updated BEFORE applying the move to maintain sync
        
        match mv {
            Move::Normal { from, to, capture, .. } => {
                // Update the moving piece's square in the list
                let pieces = if self.chess.turn() == Color::White {
                    &mut self.white_pieces
                } else {
                    &mut self.black_pieces
                };
                
                // Find the piece at 'from' and update to 'to'
                if let Some(idx) = pieces.iter().position(|&sq| sq == *from) {
                    pieces[idx] = *to;
                }
                
                // Handle capture: swap with last, then pop (like C++)
                if capture.is_some() {
                    let opp_pieces = if self.chess.turn() == Color::White {
                        &mut self.black_pieces
                    } else {
                        &mut self.white_pieces
                    };
                    
                    if let Some(cap_idx) = opp_pieces.iter().position(|&sq| sq == *to) {
                        let last_idx = opp_pieces.len() - 1;
                        if cap_idx != last_idx {
                            opp_pieces.swap(cap_idx, last_idx);
                        }
                        opp_pieces.pop();
                    }
                }
            }
            Move::EnPassant { from, to } => {
                // Update the moving piece's square in the list
                let pieces = if self.chess.turn() == Color::White {
                    &mut self.white_pieces
                } else {
                    &mut self.black_pieces
                };
                
                if let Some(idx) = pieces.iter().position(|&sq| sq == *from) {
                    pieces[idx] = *to;
                }
                
                // En passant: captured pawn is behind the destination
                let opp_pieces = if self.chess.turn() == Color::White {
                    &mut self.black_pieces
                } else {
                    &mut self.white_pieces
                };
                
                let capture_sq = if self.chess.turn() == Color::White {
                    Square::from_coords(to.file(), shakmaty::Rank::Fifth)
                } else {
                    Square::from_coords(to.file(), shakmaty::Rank::Fourth)
                };
                
                if let Some(cap_idx) = opp_pieces.iter().position(|&sq| sq == capture_sq) {
                    let last_idx = opp_pieces.len() - 1;
                    if cap_idx != last_idx {
                        opp_pieces.swap(cap_idx, last_idx);
                    }
                    opp_pieces.pop();
                }
            }
            Move::Castle { king, rook } => {
                let pieces = if self.chess.turn() == Color::White {
                    &mut self.white_pieces
                } else {
                    &mut self.black_pieces
                };
                
                // Determine new positions based on castling side
                let (new_king, new_rook) = if rook.file() == shakmaty::File::H {
                    // Kingside: king e→g, rook h→f
                    (Square::from_coords(shakmaty::File::G, king.rank()),
                     Square::from_coords(shakmaty::File::F, king.rank()))
                } else {
                    // Queenside: king e→c, rook a→d  
                    (Square::from_coords(shakmaty::File::C, king.rank()),
                     Square::from_coords(shakmaty::File::D, king.rank()))
                };
                
                // Update both king and rook
                for sq in pieces.iter_mut() {
                    if *sq == *king {
                        *sq = new_king;
                    } else if *sq == *rook {
                        *sq = new_rook;
                    }
                }
            }
            _ => {}
        }
        
        // Apply the move to the chess position
        self.chess = self.chess.clone().play(mv)
            .map_err(|e| format!("Invalid move: {:?}", e))?;
        
        Ok(())
    }
    
    pub fn get_board(&self) -> &shakmaty::Board {
        self.chess.board()
    }
    
    pub fn get_piece_list(&self) -> &[Square] {
        match self.chess.turn() {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        }
    }
    
    pub fn decode_move(&self, byte: u8, next_byte: Option<u8>) -> Result<Move, String> {
        let piece_num = byte >> 4;
        let val = byte & 15;
        
        let piece_list = self.get_piece_list();
        if (piece_num as usize) >= piece_list.len() {
            return Err(format!("Piece number {} out of range (list size: {})", 
                             piece_num, piece_list.len()));
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
                
                // Determine if this is a capture based on the move direction
                // val 1,4,7,10,13: straight forward (no capture)
                // val 15: double push forward (no capture)
                // val 0,2,3,5,6,8,9,11,12,14: diagonal (capture or en passant)
                let is_diagonal = match val {
                    1 | 4 | 7 | 10 | 13 | 15 => false,  // Forward moves
                    _ => true,  // Diagonal moves
                };
                
                let capture = if is_diagonal {
                    // Diagonal move: check for piece at destination or en passant
                    if let Some(p) = self.chess.board().piece_at(to) {
                        Some(p.role)
                    } else if self.chess.ep_square(EnPassantMode::Legal) == Some(to) {
                        Some(Role::Pawn)  // En passant
                    } else {
                        None  // Invalid diagonal without capture
                    }
                } else {
                    None  // Forward push, no capture
                };
                
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
        
        // Regular move
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
        // C++ decodeKing uses: { 0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2 }
        const KING_OFFSETS: [i8; 11] = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];
        
        if val >= 11 {
            return Err(format!("Invalid king move value: {}", val));
        }
        
        let offset = KING_OFFSETS[val as usize];
        let new_idx = (from as i8) + offset;
        
        if new_idx < 0 || new_idx >= 64 {
            return Err(format!("King move out of bounds"));
        }
        
        Square::try_from(new_idx as u8)
            .map_err(|_| format!("Invalid square: {}", new_idx))
    }
    
    fn decode_queen(&self, from: Square, val: u8, next_byte: Option<u8>) -> Result<Square, String> {
        let from_file = from.file() as u8;
        
        if val == from_file {
            // Diagonal move - two byte encoding
            let byte2 = next_byte.ok_or("Queen diagonal move requires second byte")?;
            if byte2 < 64 {
                return Err(format!("Invalid queen diagonal second byte: {}", byte2));
            }
            let to_idx = byte2 - 64;
            Square::try_from(to_idx)
                .map_err(|_| format!("Invalid queen diagonal square: {}", to_idx))
        } else {
            // Rook-like move (file or rank)
            self.decode_rook(from, val)
        }
    }
    
    fn decode_rook(&self, from: Square, val: u8) -> Result<Square, String> {
        let from_idx = from as u8;
        let from_rank = from_idx / 8;
        let from_file = from_idx % 8;
        
        let to_idx = if val < 8 {
            // Same rank, different file
            from_rank * 8 + val
        } else {
            // Same file, different rank
            (val - 8) * 8 + from_file
        };
        
        Square::try_from(to_idx)
            .map_err(|_| format!("Invalid rook square: {}", to_idx))
    }
    
    fn decode_bishop(&self, from: Square, val: u8) -> Result<Square, String> {
        // C++ decodeBishop logic:
        // byte fyle = (val & 7);
        // int fylediff = (int)fyle - (int)square_Fyle(sm->from);
        // if (val >= 8) {
        //     sm->to = sm->from - 7 * fylediff;  // up-left/down-right
        // } else {
        //     sm->to = sm->from + 9 * fylediff;  // down-left/up-right
        // }
        
        let target_file = (val & 7) as i8;
        let from_file = (from.file() as i8);
        let file_diff = target_file - from_file;
        
        let from_idx = from as i8;
        let to_idx = if val >= 8 {
            // up-left/down-right direction
            from_idx - 7 * file_diff
        } else {
            // down-left/up-right direction  
            from_idx + 9 * file_diff
        };
        
        if to_idx < 0 || to_idx >= 64 {
            return Err(format!("Bishop move out of bounds: from={}, to={}, val={}", 
                             from, to_idx, val));
        }
        
        Square::try_from(to_idx as u8)
            .map_err(|_| format!("Invalid bishop square: {}", to_idx))
    }
    
    fn decode_knight(&self, from: Square, val: u8) -> Result<Square, String> {
        // C++ decodeKnight uses: { 0, -17, -15, -10, -6, 6, 10, 15, 17 }
        const KNIGHT_OFFSETS: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];
        
        if val >= 9 {
            return Err(format!("Invalid knight move value: {}", val));
        }
        
        let offset = KNIGHT_OFFSETS[val as usize];
        let new_idx = (from as i8) + offset;
        
        if new_idx < 0 || new_idx >= 64 {
            return Err(format!("Knight move out of bounds"));
        }
        
        Square::try_from(new_idx as u8)
            .map_err(|_| format!("Invalid square: {}", new_idx))
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
