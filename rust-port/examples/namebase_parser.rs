// Complete SCID namebase (.sn4) parser
// Implements front-coded string decoding and frequency data

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const NAMEBASE_MAGIC: &[u8] = b"Scid.sn";

#[derive(Debug)]
pub struct NameBase {
    pub players: Vec<String>,
    pub events: Vec<String>,
    pub sites: Vec<String>,
    pub rounds: Vec<String>,
}

#[derive(Debug)]
struct NameBaseHeader {
    magic: [u8; 8],
    timestamp: u32,
    num_names: [u32; 4],      // [players, events, sites, rounds]
    max_frequency: [u32; 4],
}

impl NameBase {
    pub fn read_from_file(base_path: &str) -> Result<Self, String> {
        let path = format!("{}.sn4", base_path);
        let mut file = File::open(&path)
            .map_err(|e| format!("Cannot open namebase file {}: {}", path, e))?;

        // Read header
        let header = Self::read_header(&mut file)?;
        
        // Read all name types
        let players = Self::read_name_type(&mut file, 0, &header)?;
        let events = Self::read_name_type(&mut file, 1, &header)?;
        let sites = Self::read_name_type(&mut file, 2, &header)?;
        let rounds = Self::read_name_type(&mut file, 3, &header)?;

        Ok(NameBase {
            players,
            events,
            sites,
            rounds,
        })
    }

    fn read_header(file: &mut File) -> Result<NameBaseHeader, String> {
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .map_err(|e| format!("Cannot read magic: {}", e))?;
        
        if &magic[..7] != NAMEBASE_MAGIC {
            return Err(format!("Invalid magic: expected Scid.sn, got {:?}", 
                             String::from_utf8_lossy(&magic)));
        }

        let timestamp = Self::read_u32(file)?;
        
        let mut num_names = [0u32; 4];
        for i in 0..4 {
            num_names[i] = Self::read_u24(file)?;
        }
        
        let mut max_frequency = [0u32; 4];
        for i in 0..4 {
            max_frequency[i] = Self::read_u24(file)?;
        }

        Ok(NameBaseHeader {
            magic,
            timestamp,
            num_names,
            max_frequency,
        })
    }

    fn read_name_type(
        file: &mut File,
        name_type: usize,
        header: &NameBaseHeader,
    ) -> Result<Vec<String>, String> {
        let num_names = header.num_names[name_type];
        let max_freq = header.max_frequency[name_type];
        
        // Find the maximum ID to size our array
        let max_id = if name_type == 0 { 1048575 } else if name_type == 3 { 262143 } else { 524287 };
        let mut names = vec![String::new(); max_id.min(num_names as usize + 1000) as usize];
        let mut prev_name = String::new();

        for i in 0..num_names {
            // Read ID (2 or 3 bytes depending on count)
            let id = if num_names >= 65536 {
                Self::read_u24(file)?
            } else {
                Self::read_u16(file)? as u32
            };

            // Read frequency (1, 2, or 3 bytes depending on max)
            let _frequency = if max_freq >= 65536 {
                Self::read_u24(file)?
            } else if max_freq >= 256 {
                Self::read_u16(file)? as u32
            } else {
                Self::read_u8(file)? as u32
            };

            // Read name string (front-coded)
            let length = Self::read_u8(file)? as usize;
            
            let prefix = if i > 0 {
                Self::read_u8(file)? as usize
            } else {
                0
            };

            // Build the name from prefix + suffix
            let mut name = String::new();
            if prefix > 0 && prefix <= prev_name.len() {
                name.push_str(&prev_name[..prefix]);
            }
            
            let suffix_len = length.saturating_sub(prefix);
            if suffix_len > 0 {
                let mut suffix = vec![0u8; suffix_len];
                file.read_exact(&mut suffix)
                    .map_err(|e| format!("Cannot read name suffix: {}", e))?;
                name.push_str(&String::from_utf8_lossy(&suffix));
            }

            // Store by ID, not by order
            if (id as usize) < names.len() {
                names[id as usize] = name.clone();
            }
            prev_name = name;
        }

        Ok(names)
    }

    fn read_u8(file: &mut File) -> Result<u8, String> {
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Cannot read u8: {}", e))?;
        Ok(buf[0])
    }

    fn read_u16(file: &mut File) -> Result<u16, String> {
        let mut buf = [0u8; 2];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Cannot read u16: {}", e))?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_u24(file: &mut File) -> Result<u32, String> {
        let mut buf = [0u8; 3];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Cannot read u24: {}", e))?;
        Ok(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32))
    }

    fn read_u32(file: &mut File) -> Result<u32, String> {
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Cannot read u32: {}", e))?;
        Ok(u32::from_be_bytes(buf))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_path = if args.len() > 1 {
        &args[1]
    } else {
        "../bases/matein1"
    };

    match NameBase::read_from_file(base_path) {
        Ok(nb) => {
            println!("Namebase for: {}", base_path);
            println!("Players: {} entries", nb.players.len());
            println!("Events: {:?}", nb.events);
            println!("Sites: {:?}", nb.sites);
            println!("Rounds: {:?}", nb.rounds);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
