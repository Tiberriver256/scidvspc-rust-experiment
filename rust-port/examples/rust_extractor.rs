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
    if data.len() < 2 {
        return None;
    }
    
    // Skip tags using same logic as extract_moves
    const MAX_TAG_LEN: u8 = 240;
    
    let mut pos = 0;
    let mut b = data[pos];
    while b != 0 && pos < data.len() {
        pos += 1;
        if b == 255 {
            pos += 3;
        } else if b > MAX_TAG_LEN {
            if pos >= data.len() {
                return None;
            }
            let val_len = data[pos];
            pos += 1 + val_len as usize;
        } else {
            pos += b as usize;
            if pos >= data.len() {
                return None;
            }
            let val_len = data[pos];
            pos += 1 + val_len as usize;
        }
        if pos >= data.len() {
            return None;
        }
        b = data[pos];
    }
    
    // pos now points to the 0 byte ending tags
    pos += 1; // Skip the 0 byte
    
    if pos >= data.len() {
        return None;
    }
    
    // Read flags byte
    let flags = data[pos];
    pos += 1;
    
    let has_custom_fen = (flags & 1) != 0;
    
    if !has_custom_fen {
        return None; // No custom FEN
    }
    
    // Find FEN end (null-terminated)
    let fen_start = pos;
    while pos < data.len() && data[pos] != 0 {
        pos += 1;
    }
    
    if pos >= data.len() {
        return None;
    }
    
    let fen_bytes = &data[fen_start..pos];
    let fen_str = String::from_utf8_lossy(fen_bytes);
    
    // Validate it's FEN
    if fen_str.contains('/') && 
       (fen_str.contains(" w ") || fen_str.contains(" b ")) {
        Some(fen_str.into_owned())
    } else {
        None
    }
}


fn extract_moves(data: &[u8], _gnum: usize, fen: &str) -> Option<String> {
    // Format: <optional tags> <flags byte> <optional FEN\0> <moves> 0x0F
    
    if data.len() < 2 {
        return None;
    }
    
    // Skip tags by finding the byte with value 0 (end of tags)
    // Tag format: if byte > 240, it's a common tag (byte - 241 = index)
    //             if byte == 255, special 3-byte EventDate
    //             else byte = tag name length
    const MAX_TAG_LEN: u8 = 240;
    
    let mut pos = 0;
    let mut b = data[pos];
    while b != 0 && pos < data.len() {
        pos += 1;
        if b == 255 {
            // Special 3-byte binary encoding of EventDate
            pos += 3;
        } else if b > MAX_TAG_LEN {
            // Common tag: just skip the value
            if pos >= data.len() {
                return None;
            }
            let val_len = data[pos];
            pos += 1 + val_len as usize;
        } else {
            // Custom tag: skip tag name then value
            pos += b as usize;
            if pos >= data.len() {
                return None;
            }
            let val_len = data[pos];
            pos += 1 + val_len as usize;
        }
        if pos >= data.len() {
            return None;
        }
        b = data[pos];
    }
    
    // pos now points to the 0 byte ending tags
    pos += 1; // Skip the 0 byte
    
    if pos >= data.len() {
        return None;
    }
    
    // Read flags byte
    let flags = data[pos];
    pos += 1;
    
    let has_custom_fen = (flags & 1) != 0;
    
    // Read FEN if present
    let (fen_to_use, starting_move_num) = if has_custom_fen {
        // Find FEN end (null-terminated)
        let fen_start = pos;
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        
        if pos >= data.len() {
            return None;
        }
        
        let fen_bytes = &data[fen_start..pos];
        let fen_str = String::from_utf8_lossy(fen_bytes);
        
        // Extract starting move number from FEN
        let move_num = fen_str.split_whitespace().last()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1);
        
        pos += 1; // Skip null terminator
        
        (fen_str.into_owned(), move_num)
    } else {
        (fen.to_string(), 1)
    };
    
    if pos >= data.len() || data[pos] == ENCODE_END_GAME {
        return None;  // No moves
    }
    
    // Create position from FEN
    let mut decoder = MoveDecoder::from_fen(&fen_to_use).ok()?;
    let is_white = fen_to_use.contains(" w ");
    
    // Parse moves with variations, NAGs, and comments
    let mut output = Vec::new();
    let mut comment_positions = Vec::new();
    match decode_variation(data, pos, &mut decoder, is_white, starting_move_num, &mut output, &mut comment_positions, false) {
        Ok(comments_start_pos) => {
            // Read comments from end of game data
            if !comment_positions.is_empty() {
                let comments = read_comments(data, comments_start_pos, comment_positions.len());
                insert_comments(&mut output, &comment_positions, &comments);
            }
            
            if output.is_empty() {
                None
            } else {
                Some(output.join(" "))
            }
        }
        Err(e) => {
            eprintln!("Error decoding moves: {}", e);
            None
        }
    }
}

fn decode_variation(
    data: &[u8],
    start_pos: usize,
    decoder: &mut MoveDecoder,
    mut is_white: bool,
    mut move_num: usize,
    output: &mut Vec<String>,
    comment_positions: &mut Vec<usize>,
    initial_force_move_number: bool,
) -> Result<usize, String> {
    let mut pos = start_pos;
    let mut decoder_before_last_move: Option<MoveDecoder> = None;
    let mut force_move_number = initial_force_move_number; // Set to true after variations
    
    while pos < data.len() {
        let byte_val = data[pos];
        
        // Check for end markers
        if byte_val == ENCODE_END_GAME || byte_val == 14 { // END_MARKER
            pos += 1;
            break;
        }
        
        // Handle special tokens
        if byte_val >= 11 && byte_val <= 15 {
            match byte_val {
                11 => { // NAG - output immediately after last move
                    pos += 1;
                    if pos < data.len() {
                        let nag = data[pos];
                        output.push(format!("${}", nag));
                        pos += 1;
                    }
                }
                12 => { // COMMENT - mark position for later
                    pos += 1;
                    comment_positions.push(output.len());
                }
                13 => { // START_MARKER - variation
                    pos += 1;
                    
                    // Use decoder state from BEFORE the last move
                    let mut var_decoder = if let Some(ref saved) = decoder_before_last_move {
                        saved.clone()
                    } else {
                        decoder.clone()
                    };
                    
                    // Variation starts with the side to move BEFORE the last move
                    let var_is_white = !is_white;
                    
                    // Variation move number: if we just played black's move (is_white is now true),
                    // the variation shows the same move number. If we just played white's move
                    // (is_white is now false), the variation also uses the same move number.
                    // The key is: we're at position BEFORE last move was made.
                    // If is_white is currently false, we just played white's move (move_num N)
                    // So variation starts with N... for black
                    // If is_white is currently true, we just played black's move (move_num N was incremented)
                    // So variation starts with (N-1)... for black
                    let var_move_num = if is_white {
                        // We just finished black's move and incremented move_num
                        // Variation is alternative to black's move, so use previous number
                        move_num - 1
                    } else {
                        // We just finished white's move, move_num wasn't incremented yet
                        // Variation is alternative to white's move
                        move_num
                    };
                    
                    // Parse variation with position BEFORE last move
                    let mut var_output = Vec::new();
                    pos = decode_variation(data, pos, &mut var_decoder, var_is_white, var_move_num, &mut var_output, comment_positions, true)?;
                    
                    // Output variation in parentheses
                    if !var_output.is_empty() {
                        output.push(format!("( {} )", var_output.join(" ")));
                    }
                    
                    // After variation, force next move to show move number
                    force_move_number = true;
                }
                15 => { // END_GAME
                    pos += 1;
                    break;
                }
                _ => {
                    pos += 1;
                }
            }
            continue;
        }
        
        // Regular move - save decoder state BEFORE making the move
        decoder_before_last_move = Some(decoder.clone());
        
        let piece_num = byte_val >> 4;
        let val = byte_val & 15;
        
        // Determine if this needs a second byte
        let piece_list = decoder.get_piece_list();
        let needs_second_byte = if (piece_num as usize) < piece_list.len() {
            let from_sq = piece_list[piece_num as usize];
            if let Some(piece) = decoder.get_board().piece_at(from_sq) {
                // Queen diagonal move: val == from_file
                piece.role == shakmaty::Role::Queen && val as u8 == from_sq.file() as u8
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
                decoder.apply_move(&mv).map_err(|e| format!("Failed to apply move: {}", e))?;
                
                let bytes_consumed = if needs_second_byte { 2 } else { 1 };
                pos += bytes_consumed;
                
                // Format with move number
                let mut move_str = String::new();
                if is_white {
                    move_str.push_str(&format!("{}.{}", move_num, san));
                } else {
                    // Black's move: show "N..." if first move, after variation, or when needed
                    if move_num == 1 && output.is_empty() || force_move_number {
                        move_str.push_str(&format!("{}...{}", move_num, san));
                    } else {
                        move_str.push_str(&san);
                    }
                }
                
                output.push(move_str);
                force_move_number = false;
                
                if !is_white {
                    move_num += 1;
                }
                is_white = !is_white;
            }
            Err(e) => {
                return Err(format!("Failed to decode move at pos {}: {}", pos, e));
            }
        }
    }
    
    Ok(pos)
}

fn read_comments(data: &[u8], start_pos: usize, num_comments: usize) -> Vec<String> {
    let mut comments = Vec::new();
    let mut pos = start_pos;
    
    for _ in 0..num_comments {
        if pos >= data.len() {
            break;
        }
        
        // Read null-terminated string
        let comment_start = pos;
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        
        let comment_bytes = &data[comment_start..pos];
        let comment = String::from_utf8_lossy(comment_bytes).to_string();
        comments.push(comment);
        
        if pos < data.len() {
            pos += 1; // Skip null terminator
        }
    }
    
    comments
}

fn insert_comments(output: &mut Vec<String>, positions: &[usize], comments: &[String]) {
    // Insert comments at specified positions in reverse order to maintain indices
    for (comment_idx, &output_idx) in positions.iter().enumerate().rev() {
        if comment_idx < comments.len() && output_idx <= output.len() {
            output.insert(output_idx, format!("{{{}}}", comments[comment_idx]));
        }
    }
}
