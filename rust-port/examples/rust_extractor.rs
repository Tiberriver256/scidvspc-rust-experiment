// Rust extractor - reads SCID database and outputs PGN
// Should produce IDENTICAL output to C++ oracle

mod move_decoder;
mod namebase_parser;
mod tag_decoder;

use move_decoder::MoveDecoder;
use namebase_parser::NameBase;
use tag_decoder::TagDecoder;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const ENCODE_END_GAME: u8 = 15;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <database> <game_numbers...>", args[0]);
        eprintln!("Example: {} bases/matein1 1 100 500", args[0]);
        return Ok(());
    }
    
    let dbname = &args[1];
    let game_numbers: Vec<usize> = args[2..].iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    
    let db_path = dbname;  // Used for namebase parser
    
    // Open database files
    let sg4_path = format!("{}.sg4", dbname);  // Games
    let si4_path = format!("{}.si4", dbname);  // Index
    
    let mut game_file = File::open(&sg4_path)?;
    let mut index_file = File::open(&si4_path)?;
    
    eprintln!("Database: {}", dbname);
    
    // Read index to find game offsets
    let index = read_index(&mut index_file)?;
    eprintln!("Total games: {}", index.num_games);
    eprintln!();
    
    // Read namebase
    let names = read_namebase(&db_path)?;
    
    // Extract each game
    for gnum in game_numbers {
        if gnum < 1 || gnum > index.num_games {
            eprintln!("Warning: Game {} out of range", gnum);
            continue;
        }
        
        let entry = &index.entries[gnum - 1];
        
        eprintln!("Game {}: offset={}, length={}, site_id={}",
                 gnum, entry.offset, entry.length, entry.site_id);
        
        // Read game data
        game_file.seek(SeekFrom::Start(entry.offset as u64))?;
        let mut game_data = vec![0u8; entry.length];
        game_file.read_exact(&mut game_data)?;
        
        // DEBUG: print first bytes
        if gnum == 1 || gnum == 100 {
            eprint!("Game {} first 50 bytes: ", gnum);
            for i in 0..50.min(game_data.len()) {
                eprint!("{:02x} ", game_data[i]);
            }
            eprintln!();
        }
        
        // Parse and output PGN
        let pgn = extract_pgn(&game_data, entry, &names, &index, gnum)?;
        
        println!("### GAME {} ###", gnum);
        println!("{}", pgn);
        println!("### END GAME {} ###", gnum);
        println!();
    }
    
    Ok(())
}

#[derive(Debug)]
struct IndexHeader {
    magic: [u8; 8],
    version: u16,
    num_games: usize,
}

#[derive(Debug)]
struct IndexEntry {
    offset: u32,
    length: usize,
    white_id: u32,
    black_id: u32,
    event_id: u32,
    site_id: u32,
    round_id: u32,
    date_year: u32,
    date_month: u32,
    date_day: u32,
    event_date_year: u32,
    event_date_month: u32,
    event_date_day: u32,
    result: u8,  // 0=none, 1=white win, 2=black win, 3=draw
}

#[derive(Debug)]
struct Index {
    num_games: usize,
    entries: Vec<IndexEntry>,
}

fn read_index(file: &mut File) -> Result<Index, Box<dyn std::error::Error>> {
    let mut header = vec![0u8; 182]; // Index header size
    file.read_exact(&mut header)?;
    
    // Parse header: magic(8), version(2, BE!), baseType(4), numGames(3), autoLoad(3)
    let magic = &header[0..8];
    let version = u16::from_be_bytes([header[8], header[9]]);  // BIG ENDIAN
    let base_type = u32::from_be_bytes([header[10], header[11], header[12], header[13]]);  // BIG ENDIAN
    // NumGames is 3 bytes (24-bit), big endian
    let num_games = (((header[14] as u32) << 16) | 
                     ((header[15] as u32) << 8) | 
                     (header[16] as u32)) as usize;
    
    eprintln!("Index magic: {:?}", std::str::from_utf8(magic).unwrap_or("?"));
    eprintln!("Index version: {}", version);
    eprintln!("Base type: {}", base_type);
    
    // Read index entries (47 bytes each in v4)
    // Order from IndexEntry::Read():
    // Offset (4), Length_Low (2), Length_High (1), Flags (2),
    // WhiteBlack_High (1), WhiteID_Low (2), BlackID_Low (2),
    // EventSiteRnd_High (1), EventID_Low (2), SiteID_Low (2), RoundID_Low (2),
    // VarCounts (2), EcoCode (2), Dates (4), WhiteElo (2), BlackElo (2),
    // FinalMatSig (4), NumHalfMoves (1), HomePawnData (9)
    
    let entry_size = 47;
    let mut entries = Vec::new();
    
    for i in 0..num_games {
        let mut entry_data = vec![0u8; entry_size];
        file.read_exact(&mut entry_data)?;
        
        // All multi-byte values are BIG ENDIAN
        let mut pos = 0;
        
        // Offset (4 bytes)
        let offset = u32::from_be_bytes([
            entry_data[pos], entry_data[pos+1], entry_data[pos+2], entry_data[pos+3]
        ]);
        pos += 4;
        
        // Length_Low (2 bytes)
        let length_low = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // Length_High (1 byte)
        let length_high = entry_data[pos];
        pos += 1;
        
        // Length calculation: Length_Low + ((Length_High & 0x80) << 9)
        let length = (length_low as u32 + ((length_high as u32 & 0x80) << 9)) as usize;
        
        // Flags (2 bytes)
        let _flags = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // WhiteBlack_High (1 byte)
        let white_black_high = entry_data[pos];
        pos += 1;
        
        // WhiteID_Low (2 bytes)
        let white_id_low = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // BlackID_Low (2 bytes)
        let black_id_low = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // EventSiteRnd_High (1 byte)
        let event_site_rnd_high = entry_data[pos];
        pos += 1;
        
        // EventID_Low (2 bytes)
        let event_id_low = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // SiteID_Low (2 bytes)
        let site_id_low = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // RoundID_Low (2 bytes)
        let round_id_low = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // VarCounts (2 bytes) - bits 12-15 contain result
        let var_counts = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        let result = ((var_counts >> 12) & 0x0F) as u8;
        
        // EcoCode (2 bytes)
        let _eco_code = u16::from_be_bytes([entry_data[pos], entry_data[pos+1]]);
        pos += 2;
        
        // Dates (4 bytes) - contains Date in bits 0-19, EventDate in bits 20-31
        let dates = u32::from_be_bytes([
            entry_data[pos], entry_data[pos+1], entry_data[pos+2], entry_data[pos+3]
        ]);
        pos += 4;
        
        // Extract date from bits 0-19: YYYY*512 + MM*32 + DD
        let date = dates & 0xFFFFF;  // Low 20 bits
        let date_year = date / 512;
        let date_month = (date % 512) / 32;
        let date_day = date % 32;
        
        // EventDate is in bits 20-31 (high 12 bits)
        let event_date_raw = (dates >> 20) & 0xFFF;  // 12 bits
        let event_date_month = (event_date_raw % 512) / 32;
        let event_date_day = event_date_raw % 32;
        // Year is stored as 3-bit offset (0-7) representing (-4 to +3) from game year
        let year_offset = (event_date_raw / 512) & 7;
        let event_date_year = if year_offset == 0 {
            0  // ZERO_DATE
        } else {
            (date_year as i32 + year_offset as i32 - 4) as u32
        };
        
        // Reconstruct full IDs from high/low bytes
        let white_id = ((white_black_high as u32 & 0xF0) << 12) | (white_id_low as u32);
        let black_id = ((white_black_high as u32 & 0x0F) << 16) | (black_id_low as u32);
        let event_id = ((event_site_rnd_high as u32 & 0xF0) << 12) | (event_id_low as u32);
        let site_id = ((event_site_rnd_high as u32 & 0x0C) << 14) | (site_id_low as u32);
        let round_id = ((event_site_rnd_high as u32 & 0x03) << 16) | (round_id_low as u32);
        
        if i < 3 {
            eprintln!("Entry {}: offset={}, length={}, white={}, black={}, event={}, site={}, round={}, result={}, date={}/{}/{}",
                     i+1, offset, length, white_id, black_id, event_id, site_id, round_id, result, date_year, date_month, date_day);
        }
        
        entries.push(IndexEntry {
            offset,
            length,
            white_id,
            black_id,
            event_id,
            site_id,
            round_id,
            date_year,
            date_month,
            date_day,
            event_date_year,
            event_date_month,
            event_date_day,
            result,
        });
    }
    
    Ok(Index { num_games, entries })
}

fn read_namebase(db_path: &str) -> Result<NameBase, Box<dyn std::error::Error>> {
    // Use the proper namebase parser
    match NameBase::read_from_file(db_path) {
        Ok(nb) => Ok(nb),
        Err(e) => {
            eprintln!("Warning: Cannot read namebase: {}", e);
            // Return minimal namebase
            Ok(NameBase {
                players: vec!["?".to_string()],
                events: vec!["?".to_string()],
                sites: vec!["?".to_string()],
                rounds: vec!["?".to_string()],
            })
        }
    }
}

fn extract_pgn(
    game_data: &[u8],
    entry: &IndexEntry,
    names: &NameBase,
    index: &Index,
    gnum: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut pgn = String::new();
    
    // Decode tags first
    let mut tag_decoder = TagDecoder::new();
    let tags_end = tag_decoder.decode_tags(game_data).unwrap_or(0);
    
    // Get names
    let event = names.events.get(entry.event_id as usize).unwrap_or(&names.events[0]);
    let site = names.sites.get(entry.site_id as usize).unwrap_or(&names.sites[0]);
    let white = names.players.get(entry.white_id as usize).unwrap_or(&names.players[0]);
    let black = names.players.get(entry.black_id as usize).unwrap_or(&names.players[0]);
    let round = names.rounds.get(entry.round_id as usize).unwrap_or(&names.rounds[0]);
    
    // Get date from index
    let year = entry.date_year;
    let month = entry.date_month;
    let day = entry.date_day;
    let date_str = if year > 0 {
        format!("{:04}.{}.{}", year, 
               if month > 0 { format!("{:02}", month) } else { "??".to_string() },
               if day > 0 { format!("{:02}", day) } else { "??".to_string() })
    } else {
        "????.??.??".to_string()
    };
    
    // Get result from index
    let result_str = match entry.result {
        1 => "1-0",
        2 => "0-1",
        3 => "1/2-1/2",
        _ => "*",
    };
    
    // Write standard tags
    pgn.push_str(&format!("[Event \"{}\"]\n", event));
    pgn.push_str(&format!("[Site \"{}\"]\n", site));
    pgn.push_str(&format!("[Date \"{}\"]\n", date_str));
    pgn.push_str(&format!("[Round \"{}\"]\n", round));
    pgn.push_str(&format!("[White \"{}\"]\n", white));
    pgn.push_str(&format!("[Black \"{}\"]\n", black));
    pgn.push_str(&format!("[Result \"{}\"]\n", result_str));
    
    // Add EventDate if present in tags, otherwise from index
    if let Some(event_date) = tag_decoder.get("EventDate") {
        pgn.push_str(&format!("[EventDate \"{}\"]\n", event_date));
    } else if entry.event_date_year > 0 {
        let ed_str = format!("{:04}.{}.{}", entry.event_date_year,
               if entry.event_date_month > 0 { format!("{:02}", entry.event_date_month) } else { "??".to_string() },
               if entry.event_date_day > 0 { format!("{:02}", entry.event_date_day) } else { "??".to_string() });
        pgn.push_str(&format!("[EventDate \"{}\"]\n", ed_str));
    }
    
    // Add Annotator if present
    if let Some(annotator) = tag_decoder.get("Annotator") {
        pgn.push_str(&format!("[Annotator \"{}\"]\n", annotator));
    }
    
    pgn.push_str("[SetUp \"1\"]\n");
    
    // Add PlyCount if present in tags
    if let Some(plycount) = tag_decoder.get("PlyCount") {
        pgn.push_str(&format!("[PlyCount \"{}\"]\n", plycount));
    } else {
        pgn.push_str("[PlyCount \"0\"]\n");
    }
    
    // Extract FEN
    if let Some(fen) = extract_fen(game_data) {
        pgn.push_str(&format!("[FEN \"{}\"]\n", fen));
    }
    
    pgn.push_str("\n");
    
    // Extract moves (need FEN for position setup)
    let fen_str = extract_fen(game_data).unwrap_or_default();
    if let Some(moves) = extract_moves(game_data, gnum, &fen_str) {
        pgn.push_str(&moves);
        pgn.push_str(" ");
    }
    
    pgn.push_str(result_str);
    pgn.push_str("\n");
    
    Ok(pgn)
}

fn extract_fen(data: &[u8]) -> Option<String> {
    // Format: <optional tags> 0xFA 0x01 <header (6 bytes)> <FEN>\0 ...
    // Need to skip tags first, then FEN starts at offset 8 from game marker
    
    if data.len() < 20 {
        return None;
    }
    
    // Find the game marker 0xFA 0x01
    let mut marker_pos = 0;
    for i in 0..data.len()-1 {
        if data[i] == 0xFA && data[i+1] == 0x01 {
            marker_pos = i;
            break;
        }
    }
    
    if marker_pos == 0 && (data[0] != 0xFA || data[1] != 0x01) {
        return None;  // No marker found
    }
    
    // FEN starts 8 bytes after the marker
    let fen_start = marker_pos + 8;
    
    if fen_start >= data.len() {
        return None;
    }
    
    // Find the end (null terminator)
    let mut fen_end = fen_start;
    while fen_end < data.len() && data[fen_end] != 0 {
        fen_end += 1;
    }
    
    if fen_end <= fen_start || fen_end >= data.len() {
        return None;
    }
    
    if let Ok(fen_candidate) = std::str::from_utf8(&data[fen_start..fen_end]) {
        // Validate it's really FEN (should have slashes and "w" or "b")
        if fen_candidate.contains('/') && 
           (fen_candidate.contains(" w ") || fen_candidate.contains(" b ")) {
            return Some(fen_candidate.to_string());
        }
    }
    
    None
}

fn extract_moves(data: &[u8], gnum: usize, fen: &str) -> Option<String> {
    // Format: <optional tags> 0xFA 0x01 <header (6 bytes)> <FEN>\0 <moves> 0x0F
    // Need to find game marker first
    
    if data.len() < 10 {
        return None;
    }
    
    // Find the game marker 0xFA 0x01
    let mut marker_pos = 0;
    for i in 0..data.len()-1 {
        if data[i] == 0xFA && data[i+1] == 0x01 {
            marker_pos = i;
            break;
        }
    }
    
    if marker_pos == 0 && (data[0] != 0xFA || data[1] != 0x01) {
        return None;  // No marker found
    }
    
    // FEN starts 8 bytes after marker
    let fen_start = marker_pos + 8;
    
    // Find the FEN end
    let mut fen_end = fen_start;
    while fen_end < data.len() && data[fen_end] != 0 {
        fen_end += 1;
    }
    
    if fen_end >= data.len() - 1 {
        return None;  // No room for moves
    }
    
    if gnum == 1 {
        eprintln!("Game 1: data.len()={}, FEN ends at byte {}", data.len(), fen_end);
        eprintln!("Next bytes:");
        for i in 0..5.min(data.len() - fen_end) {
            if fen_end + i < data.len() {
                eprintln!("  [{}]: 0x{:02x}", fen_end + i, data[fen_end + i]);
            }
        }
    }
    
    // Create position from FEN
    let mut decoder = MoveDecoder::from_fen(fen).ok()?;
    
    // Move data starts immediately after FEN null terminator
    let mut pos = fen_end + 1;
    
    if pos >= data.len() || data[pos] == ENCODE_END_GAME {
        return None;  // No moves
    }
    
    // Decode moves
    let mut moves = Vec::new();
    let mut move_num = 1;
    let mut is_white = fen.contains(" w ");
    
    while pos < data.len() {
        let byte_val = data[pos];
        
        if byte_val == ENCODE_END_GAME {
            break;
        }
        
        // Check if this is a special token (byte values 11-15)
        if byte_val >= 11 && byte_val <= 15 {
            pos += 1;
            if byte_val == 11 { // NAG
                pos += 1; // Skip NAG byte
            }
            continue;
        }
        
        // Regular move
        let piece_num = byte_val >> 4;
        let val = byte_val & 15;
        
        // Determine if this needs a second byte
        let piece_list = decoder.get_piece_list();
        let needs_second_byte = if (piece_num as usize) < piece_list.len() {
            let from_sq = piece_list[piece_num as usize];
            if let Some(piece) = decoder.get_board().piece_at(from_sq) {
                // Queen diagonal move: val == from_file
                let result = piece.role == shakmaty::Role::Queen && val as u8 == from_sq.file() as u8;
                if gnum == 1 {
                    eprintln!("  piece_num={}, val={}, piece={:?}, from_sq={}, from_file={}, needs_second={}", 
                             piece_num, val, piece.role, from_sq, from_sq.file() as u8, result);
                }
                result
            } else {
                false
            }
        } else {
            false
        };
        
        let next_byte = if needs_second_byte && pos + 1 < data.len() {
            Some(data[pos + 1])
        } else {
            None
        };
        
        match decoder.decode_move(byte_val, next_byte) {
            Ok(mv) => {
                let san = decoder.move_to_san(&mv);
                
                // Apply the move to update position for next iteration
                if let Err(e) = decoder.apply_move(&mv) {
                    eprintln!("Failed to apply move: {}", e);
                    break;
                }
                
                let bytes_consumed = if needs_second_byte { 2 } else { 1 };
                pos += bytes_consumed;
                
                // Format with move number
                if is_white {
                    moves.push(format!("{}.{}", move_num, san));
                } else {
                    if move_num == 1 && moves.is_empty() {
                        // First move is black's
                        moves.push(format!("{}...{}", move_num, san));
                    } else {
                        moves.push(san);
                    }
                    move_num += 1;
                }
                is_white = !is_white;
            }
            Err(e) => {
                eprintln!("Failed to decode move at pos {}: {}", pos, e);
                break;
            }
        }
    }
    
    if moves.is_empty() {
        None
    } else {
        Some(moves.join(" "))
    }
}
