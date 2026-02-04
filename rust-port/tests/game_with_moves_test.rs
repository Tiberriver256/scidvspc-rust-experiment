//! Integration test with a complete game including moves

use scid_pgn::{Game, PgnWriter};
use scid_pgn::game::MoveNode;
use scid_pgn::pgn::PgnOptions;
use scid_pgn::types::{GameResult, Move, Piece, PieceType, Color, Square};

#[test]
fn test_complete_game_with_moves() {
    // Create a simple game: 1. e4 e5 2. Nf3 Nc6
    let mut game = Game::new();
    game.event = "Test Event".to_string();
    game.site = "Test Site".to_string();
    game.date = "2024.02.04".to_string();
    game.round = "1".to_string();
    game.white = "Player, White".to_string();
    game.black = "Player, Black".to_string();
    game.result = GameResult::White;
    
    // Add moves
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(4, 1).unwrap(), // e2
            to: Square::new(4, 3).unwrap(),   // e4
            piece: Piece { piece_type: PieceType::Pawn, color: Color::White },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        },
        san: "e4".to_string(),
        comment: None,
        nags: vec![],
        variations: vec![],
    });
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(4, 6).unwrap(), // e7
            to: Square::new(4, 4).unwrap(),   // e5
            piece: Piece { piece_type: PieceType::Pawn, color: Color::Black },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        },
        san: "e5".to_string(),
        comment: None,
        nags: vec![],
        variations: vec![],
    });
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(6, 0).unwrap(), // g1
            to: Square::new(5, 2).unwrap(),   // f3
            piece: Piece { piece_type: PieceType::Knight, color: Color::White },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        },
        san: "Nf3".to_string(),
        comment: None,
        nags: vec![],
        variations: vec![],
    });
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(1, 7).unwrap(), // b8
            to: Square::new(2, 5).unwrap(),   // c6
            piece: Piece { piece_type: PieceType::Knight, color: Color::Black },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        },
        san: "Nc6".to_string(),
        comment: None,
        nags: vec![],
        variations: vec![],
    });
    
    // Write PGN
    let writer = PgnWriter::default();
    let pgn = writer.write(&game).unwrap();
    
    println!("Generated PGN:\n{}", pgn);
    
    // Verify structure
    assert!(pgn.contains("[Event \"Test Event\"]"));
    assert!(pgn.contains("[White \"Player, White\"]"));
    assert!(pgn.contains("1. e4 e5"));
    assert!(pgn.contains("2. Nf3 Nc6"));
    assert!(pgn.contains("1-0"));
}

#[test]
fn test_game_with_comments() {
    let mut game = Game::new();
    game.event = "Test".to_string();
    game.site = "Test".to_string();
    game.date = "2024.02.04".to_string();
    game.round = "1".to_string();
    game.white = "White".to_string();
    game.black = "Black".to_string();
    game.result = GameResult::Draw;
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(4, 1).unwrap(),
            to: Square::new(4, 3).unwrap(),
            piece: Piece { piece_type: PieceType::Pawn, color: Color::White },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        },
        san: "e4".to_string(),
        comment: Some("The most popular opening move.".to_string()),
        nags: vec![1], // !
        variations: vec![],
    });
    
    let writer = PgnWriter::default();
    let pgn = writer.write(&game).unwrap();
    
    println!("PGN with comments:\n{}", pgn);
    
    assert!(pgn.contains("e4 ! {The most popular opening move.}"));
    assert!(pgn.contains("1/2-1/2"));
}

#[test]
fn test_castling() {
    let mut game = Game::new();
    game.event = "Castling Test".to_string();
    game.site = "Test".to_string();
    game.date = "2024.02.04".to_string();
    game.round = "1".to_string();
    game.white = "White".to_string();
    game.black = "Black".to_string();
    game.result = GameResult::Unknown;
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(4, 0).unwrap(), // e1
            to: Square::new(6, 0).unwrap(),   // g1
            piece: Piece { piece_type: PieceType::King, color: Color::White },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: true,
        },
        san: "O-O".to_string(),
        comment: None,
        nags: vec![],
        variations: vec![],
    });
    
    let writer = PgnWriter::default();
    let pgn = writer.write(&game).unwrap();
    
    println!("Castling PGN:\n{}", pgn);
    
    assert!(pgn.contains("1. O-O"));
}
