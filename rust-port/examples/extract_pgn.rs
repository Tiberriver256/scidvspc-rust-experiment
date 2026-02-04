// Extract and display random games from SCID database in PGN format

use std::fs::File;
use std::io::Read;

fn main() -> std::io::Result<()> {
    // Read the matein1.sg4 file
    let mut file = File::open("../bases/matein1.sg4")?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    println!("Database: matein1.sg4");
    println!("Total size: {} bytes", data.len());
    
    // Find all game starts (0xFA 0x01 marker)
    let mut game_starts = Vec::new();
    for i in 0..data.len() - 1 {
        if data[i] == 0xFA && data[i+1] == 0x01 {
            game_starts.push(i);
        }
    }
    
    println!("Total games found: {}\n", game_starts.len());
    
    // Extract three random games (using fixed positions for reproducibility)
    let game_indices = [0, game_starts.len() / 2, game_starts.len() - 1];
    
    for (idx, &game_idx) in game_indices.iter().enumerate() {
        println!("═══════════════════════════════════════");
        println!("Game {} (index {})", idx + 1, game_idx);
        println!("═══════════════════════════════════════");
        
        let start_offset = game_starts[game_idx];
        let end_offset = if game_idx + 1 < game_starts.len() {
            game_starts[game_idx + 1]
        } else {
            data.len()
        };
        
        let game_data = &data[start_offset..end_offset];
        
        println!("Offset: {} - {}", start_offset, end_offset);
        println!("Size: {} bytes", game_data.len());
        
        // Try to parse the game structure
        if let Some(pgn) = parse_game(game_data) {
            println!("\n{}", pgn);
        } else {
            println!("\nCouldn't parse game structure");
            
            // Show raw data for debugging
            println!("\nFirst 100 bytes (hex):");
            for (i, chunk) in game_data[..100.min(game_data.len())].chunks(16).enumerate() {
                print!("{:04x}: ", i * 16);
                for byte in chunk {
                    print!("{:02x} ", byte);
                }
                println!();
            }
        }
        
        println!();
    }
    
    Ok(())
}

fn parse_game(data: &[u8]) -> Option<String> {
    if data.len() < 10 {
        return None;
    }
    
    let mut pos = 0;
    let mut pgn = String::new();
    
    // Check for game marker (0xFA 0x01)
    if data[pos] != 0xFA || data[pos + 1] != 0x01 {
        return None;
    }
    pos += 2;
    
    // Next bytes seem to be metadata
    // Skip to find the FEN string (look for '/' which appears in FEN)
    let mut fen_start = None;
    for i in pos..data.len() {
        if data[i] == b'/' {
            // Found a slash - back up to find the start of the FEN
            let mut start = i;
            while start > 0 && data[start - 1] >= 32 && data[start - 1] < 127 {
                start -= 1;
            }
            fen_start = Some(start);
            break;
        }
    }
    
    if let Some(fen_pos) = fen_start {
        // Extract FEN string (until null terminator or end)
        let mut fen_end = fen_pos;
        while fen_end < data.len() && data[fen_end] != 0 {
            fen_end += 1;
        }
        
        if let Ok(fen) = std::str::from_utf8(&data[fen_pos..fen_end]) {
            // Generate basic PGN with FEN
            pgn.push_str("[Event \"Mate in N\"]\n");
            pgn.push_str("[Site \"?\"]\n");
            pgn.push_str("[Date \"????.??.??\"]\n");
            pgn.push_str("[Round \"?\"]\n");
            pgn.push_str("[White \"?\"]\n");
            pgn.push_str("[Black \"?\"]\n");
            pgn.push_str("[Result \"*\"]\n");
            pgn.push_str(&format!("[FEN \"{}\"]\n", fen));
            pgn.push_str("[SetUp \"1\"]\n\n");
            
            // Try to decode moves (simplified - just show we found the position)
            // The actual move data comes after the FEN
            pos = fen_end + 1;
            
            // Look for move bytes (typically after 0x00 0x0F marker)
            if pos + 2 < data.len() {
                // Skip markers
                while pos < data.len() && (data[pos] == 0x00 || data[pos] == 0x0F) {
                    pos += 1;
                }
                
                // The remaining bytes are encoded moves
                // For now, just indicate there's move data
                let remaining = data.len() - pos;
                pgn.push_str(&format!("{{ {} bytes of move data }}\n\n", remaining));
            }
            
            pgn.push_str("*\n");
            
            return Some(pgn);
        }
    }
    
    None
}
