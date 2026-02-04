// SCID game tag decoder
// Decodes tags stored before the game marker

use std::collections::HashMap;

const MAX_TAG_LEN: u8 = 240;

// Common tags (241-254)
const COMMON_TAGS: &[Option<&str>] = &[
    Some("WhiteCountry"),  // 241
    Some("BlackCountry"),  // 242
    Some("Annotator"),     // 243
    Some("PlyCount"),      // 244
    Some("EventDate"),     // 245 (text encoding)
    Some("Opening"),       // 246
    Some("Variation"),     // 247
    Some("Setup"),         // 248
    Some("Source"),        // 249
    Some("SetUp"),         // 250
    None,                  // 251
    None,                  // 252
    None,                  // 253
    None,                  // 254
    None,                  // 255 (binary EventDate)
];

pub struct TagDecoder {
    pub tags: HashMap<String, String>,
}

impl TagDecoder {
    pub fn new() -> Self {
        TagDecoder {
            tags: HashMap::new(),
        }
    }

    pub fn decode_tags(&mut self, data: &[u8]) -> Result<usize, String> {
        let mut pos = 0;

        loop {
            if pos >= data.len() {
                return Err("Unexpected end of tag data".to_string());
            }

            let b = data[pos];
            pos += 1;

            if b == 0 {
                // End of tags
                break;
            }

            if b == 255 {
                // Binary EventDate (3 bytes)
                if pos + 3 > data.len() {
                    return Err("EventDate data too short".to_string());
                }
                let date = ((data[pos] as u32) << 16) 
                         | ((data[pos + 1] as u32) << 8)
                         | (data[pos + 2] as u32);
                pos += 3;

                // Decode SCID date format (YYYY*512 + MM*32 + DD)
                let year = date / 512;
                let month = (date % 512) / 32;
                let day = date % 32;

                if year > 0 {
                    let date_str = format!("{:04}.{:02}.{:02}", year, month, day);
                    self.tags.insert("EventDate".to_string(), date_str);
                }
            } else if b > MAX_TAG_LEN {
                // Common tag
                let tag_index = (b - MAX_TAG_LEN - 1) as usize;
                if tag_index < COMMON_TAGS.len() {
                    if let Some(tag_name) = COMMON_TAGS[tag_index] {
                        if pos >= data.len() {
                            return Err("Tag value length missing".to_string());
                        }
                        let value_len = data[pos] as usize;
                        pos += 1;

                        if pos + value_len > data.len() {
                            return Err("Tag value data too short".to_string());
                        }

                        let value = String::from_utf8_lossy(&data[pos..pos + value_len]).to_string();
                        pos += value_len;

                        self.tags.insert(tag_name.to_string(), value);
                    }
                }
            } else {
                // Custom tag
                let tag_len = b as usize;
                if pos + tag_len > data.len() {
                    return Err("Tag name too short".to_string());
                }

                let tag_name = String::from_utf8_lossy(&data[pos..pos + tag_len]).to_string();
                pos += tag_len;

                if pos >= data.len() {
                    return Err("Tag value length missing".to_string());
                }
                let value_len = data[pos] as usize;
                pos += 1;

                if pos + value_len > data.len() {
                    return Err("Tag value data too short".to_string());
                }

                let value = String::from_utf8_lossy(&data[pos..pos + value_len]).to_string();
                pos += value_len;

                self.tags.insert(tag_name, value);
            }
        }

        Ok(pos)
    }

    pub fn get(&self, tag: &str) -> Option<&String> {
        self.tags.get(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_annotator() {
        let data = vec![
            0xF3, // Annotator (243)
            0x03, // length 3
            b'T', b'2', b'R',
            0x00, // end of tags
        ];

        let mut decoder = TagDecoder::new();
        let pos = decoder.decode_tags(&data).unwrap();
        assert_eq!(pos, 6);
        assert_eq!(decoder.get("Annotator"), Some(&"T2R".to_string()));
    }
}
