//! Example: Converting SCID games to PGN
//! 
//! This example shows how to use the scid-pgn library to convert games
//! from SCID's binary format to PGN notation.

use scid_pgn::{Converter, Game, PgnWriter};
use scid_pgn::pgn::{PgnOptions, PgnFormat};
use scid_pgn::types::{GameResult, Color, Piece, PieceType, Square, Move};
use scid_pgn::game::MoveNode;

fn main() {
    println!("SCID to PGN Converter Examples\n");
    
    // Example 1: Create a game programmatically and output PGN
    example_create_game();
    
    // Example 2: Use the converter API
    example_converter_api();
    
    // Example 3: Different output formats
    example_formats();
}

fn example_create_game() {
    println!("═══════════════════════════════════════");
    println!("Example 1: Create Game Programmatically");
    println!("═══════════════════════════════════════\n");
    
    let mut game = Game::new();
    
    // Set game metadata
    game.event = "Example Tournament".to_string();
    game.site = "Internet".to_string();
    game.date = "2024.02.04".to_string();
    game.round = "1".to_string();
    game.white = "Player, White".to_string();
    game.black = "Player, Black".to_string();
    game.result = GameResult::White;
    game.white_elo = Some(2400);
    game.black_elo = Some(2350);
    game.eco = Some("C50".to_string());
    
    // Add moves: 1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 (Italian Game)
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
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(4, 6).unwrap(),
            to: Square::new(4, 4).unwrap(),
            piece: Piece { piece_type: PieceType::Pawn, color: Color::Black },
            captured: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        },
        san: "e5".to_string(),
        comment: Some("Symmetrical response.".to_string()),
        nags: vec![],
        variations: vec![],
    });
    
    game.moves.push(MoveNode {
        move_data: Move {
            from: Square::new(6, 0).unwrap(),
            to: Square::new(5, 2).unwrap(),
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
            from: Square::new(1, 7).unwrap(),
            to: Square::new(2, 5).unwrap(),
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
    
    // Convert to PGN
    let writer = PgnWriter::default();
    let pgn = writer.write(&game).unwrap();
    
    println!("{}", pgn);
}

fn example_converter_api() {
    println!("\n═══════════════════════════════════════");
    println!("Example 2: Using Converter API");
    println!("═══════════════════════════════════════\n");
    
    // Create a converter with options
    let converter = Converter::new()
        .include_comments(true)
        .include_variations(true);
    
    println!("Converter created with:");
    println!("  - Comments: enabled");
    println!("  - Variations: enabled");
    println!("\nReady to convert SCID binary data to PGN.");
    println!("(Would need actual SCID database file to demonstrate)");
}

fn example_formats() {
    println!("\n═══════════════════════════════════════");
    println!("Example 3: Different Output Formats");
    println!("═══════════════════════════════════════\n");
    
    let mut game = Game::new();
    game.event = "Format Test".to_string();
    game.white = "Alice".to_string();
    game.black = "Bob".to_string();
    game.result = GameResult::Draw;
    
    // Standard format
    let options = PgnOptions {
        format: PgnFormat::Plain,
        include_comments: true,
        include_variations: true,
        include_nags: true,
        short_header: false,
    };
    
    let writer = PgnWriter::new(options);
    let pgn = writer.write(&game).unwrap();
    
    println!("Standard PGN Format:\n{}", pgn);
    
    // Short header format
    let options_short = PgnOptions {
        format: PgnFormat::Plain,
        short_header: true,
        ..options
    };
    
    let writer_short = PgnWriter::new(options_short);
    let pgn_short = writer_short.write(&game).unwrap();
    
    println!("\nShort Header Format:\n{}", pgn_short);
}
