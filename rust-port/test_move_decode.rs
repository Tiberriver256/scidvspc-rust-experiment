use std::fs::File;
use std::io::Read;

fn main() {
    let mut f = File::open("../bases/matein1.sg4").unwrap();
    let mut data = Vec::new();
    f.read_to_end(&mut data).unwrap();
    
    // Game 1 starts at offset 0
    // Print first 100 bytes
    println!("First 100 bytes of game 1:");
    for i in 0..100 {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", data[i]);
        if data[i] >= 32 && data[i] < 127 {
            eprint!("{}", data[i] as char);
        } else {
            eprint!(".");
        }
    }
    println!();
    
    // Find FEN string
    let fen_start = 8;
    let mut fen_end = fen_start;
    while fen_end < data.len() && data[fen_end] != 0 {
        fen_end += 1;
    }
    
    let fen = String::from_utf8_lossy(&data[fen_start..fen_end]);
    println!("\nFEN: {}", fen);
    println!("FEN ends at byte: {}", fen_end);
    
    // Print bytes after FEN
    println!("\nBytes after FEN:");
    for i in fen_end..fen_end+20 {
        print!("{:02x} ", data[i]);
    }
    println!();
}
