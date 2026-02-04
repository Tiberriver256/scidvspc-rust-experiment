//! SCID binary format codec
//! 
//! Decodes games from SCID's compact binary format into Game structs.

mod move_decoder;
pub mod game_decoder;

pub use move_decoder::{ByteBuffer, DecodeError, decode_move};
pub use game_decoder::{decode_game, DECODE_NONE, DECODE_TAGS, DECODE_COMMENTS, DECODE_ALL};
