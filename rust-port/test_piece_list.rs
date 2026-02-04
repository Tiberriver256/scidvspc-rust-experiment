use shakmaty::Position;

fn main() {
    let fen = "8/7R/3k4/8/3KQ3/8/8/7q w - - 0 1";
    
    let setup: shakmaty::fen::Fen = fen.parse().unwrap();
    let chess: shakmaty::Chess = setup.into_position(shakmaty::CastlingMode::Standard).unwrap();
    
    let mut pieces = Vec::new();
    let color = chess.turn();
    
    // King must be first
    for sq in shakmaty::Square::ALL {
        if let Some(piece) = chess.board().piece_at(sq) {
            if piece.color == color && piece.role == shakmaty::Role::King {
                pieces.push((sq, piece.role));
                break;
            }
        }
    }
    
    // Then all other pieces
    for sq in shakmaty::Square::ALL {
        if let Some(piece) = chess.board().piece_at(sq) {
            if piece.color == color && piece.role != shakmaty::Role::King {
                pieces.push((sq, piece.role));
            }
        }
    }
    
    println!("White pieces:");
    for (i, (sq, role)) in pieces.iter().enumerate() {
        println!("  {}: {:?} at {}", i, role, sq);
    }
}
