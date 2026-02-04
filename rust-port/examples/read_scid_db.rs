// Simple test to read raw SCID database file

use std::fs::File;
use std::io::Read;

fn main() -> std::io::Result<()> {
    // Read the matein1.sg4 file
    let mut file = File::open("../bases/matein1.sg4")?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    println!("File size: {} bytes", data.len());
    println!("\nFirst 512 bytes (hex):");
    for (i, chunk) in data[..512.min(data.len())].chunks(16).enumerate() {
        print!("{:04x}: ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        print!(" | ");
        for byte in chunk {
            let c = if *byte >= 32 && *byte < 127 { *byte as char } else { '.' };
            print!("{}", c);
        }
        println!();
    }
    
    // Try to find patterns
    println!("\n\nLooking for patterns...");
    
    // Each game seems to start with 0xFA 0x01
    let mut game_starts = Vec::new();
    for i in 0..data.len() - 1 {
        if data[i] == 0xFA && data[i+1] == 0x01 {
            game_starts.push(i);
        }
    }
    
    println!("Found {} potential game starts at positions:", game_starts.len());
    for (idx, pos) in game_starts.iter().take(10).enumerate() {
        println!("  Game {}: offset {}", idx + 1, pos);
    }
    
    // Look at first game structure
    if !game_starts.is_empty() {
        let start = game_starts[0];
        println!("\nFirst game details (starting at {}):", start);
        println!("  Bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", 
                 data[start], data[start+1], data[start+2], 
                 data[start+3], data[start+4], data[start+5]);
        
        // Look for FEN string (starts after header, usually has '/' for ranks)
        for i in start..start.min(start + 200) {
            if data[i] == b'/' {
                // Found a FEN string, print context
                let start_pos = i.saturating_sub(20);
                let end_pos = (i + 60).min(data.len());
                let snippet = String::from_utf8_lossy(&data[start_pos..end_pos]);
                println!("\n  FEN found near offset {}:\n    {}", i, snippet);
                break;
            }
        }
    }
    
    Ok(())
}
