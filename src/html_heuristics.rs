use std::collections::hash_map::HashMap;
use std::char;
use std::iter;

/// Maximum number of strings allowed to be set (all lower-cased)
const MAX_STRINGS: usize = 1024;

/// Maximum number of chars to be taken into account
const MAX_CHARS: usize = 255;

lazy_static! {
    static ref ALL_TWO_CHARS: Vec<String> = {
        let mut v: Vec<String> = Vec::new();
        
        for i in 0u8..255 {
            for j in 0u8..255 {
                let ch1 = i as char;
                let ch2 = j as char;
                
                v.push(ch1.to_string() + &ch2.to_string());
            }
        }
        
        v
    };
}

/// This class will control HTML tag heuristics that will allow faster matching of tags
/// to avoid long cycles as well as creation of same strings over and over again.
/// 
/// This is effectively a fancy hash lookup table with attributes being hashed in context of tag
pub struct HtmlHeuristics {
    /// Array in which we will keep char hints to quickly match	ID (if non-zero) of tag
    chars: [[i16; 256]; 256],

    /// Strings used, once matched they will be returned to avoid creation of a brand new string
    /// and all associated costs with it
    strings: Vec<Option<String>>,

    /// Binary data represending tag strings is here: case sensitive: lower case for even even value, and odd for each odd
    /// for the same string
    tag_data: Vec<Vec<u8>>,

    /// List of added tags to avoid dups
    added_tags: HashMap<String, i16>,

    /// Hash that will contain single char mapping hash
    attributes: Vec<Vec<u8>>,

    /// Binary data represending attribute strings is here: case sensitive: lower case for even even value, and odd for each odd
    /// for the same string
    attr_data: Vec<Vec<u8>>,

    /// List of added attributes to avoid dups
    added_attributes: HashMap<String, i16>,

    attrs: Vec<Option<String>>,
}

impl HtmlHeuristics {
    pub fn new() -> HtmlHeuristics {
        let chars = [[0; 256]; 256];
        let strings = vec![None; MAX_STRINGS];
        let tag_data = iter::repeat(Vec::new()).take(MAX_STRINGS*2).collect();
        let added_tags: HashMap<String, i16> = HashMap::new();
        let attributes = iter::repeat(Vec::new()).take(MAX_STRINGS*2).collect();
        let attr_data = iter::repeat(Vec::new()).take(MAX_STRINGS*2).collect();
        let added_attributes: HashMap<String, i16> = HashMap::new();
        let attrs = vec![None; MAX_STRINGS];

        let heuristics = HtmlHeuristics {
            chars: chars,
            strings: strings,
            tag_data: tag_data,
            added_tags: added_tags,
            attributes: attributes,
            attr_data: attr_data,
            added_attributes: added_attributes,
            attrs: attrs,
        };

        heuristics
    }

    /// Returns String of i and j combination
    pub fn get_two_char_string(i: u8, j: u8) -> String {
        ALL_TWO_CHARS[(i as usize)*256 + (j as usize)].clone()
    }

    /// Returns string for ID returned by GetMatch
    pub fn get_string_by_id(&self, id: usize) -> String {
        if let Some(ref s) = self.strings[id >> 1] {
            return s.clone()
        }

        "".to_string()
    }

    pub fn get_string_data(&self, id: usize) -> &Vec<u8> {
        &self.tag_data[id]
    }

    pub fn match_tag(&self, ch1: u8, ch2: u8) -> i16 {
        self.chars[ch1 as usize][ch1 as usize]
    }

    pub fn match_attr(&self, ch: u8, tag_id: usize) -> u8 {
        self.attr_data[tag_id>>1][ch as usize]
    }

    pub fn get_attr_data(&self, attr_id: usize) -> &Vec<u8> {
        &self.attributes[attr_id]
    }

    pub fn get_attr(&self, attr_id: usize) -> String {
        if let Some(ref s) = self.attrs[attr_id >> 1] {
            return s.clone()
        }

        "".to_string()
    }

    /// Adds tag to list of tracked tags (don't add too many, if you have got multiple same first
    /// 2 chars then duplicates won't be added, so make sure the first added tags are the MOST LIKELY to be found)
    pub fn add_tag(&mut self, tag_name: String, attr_names: String) -> bool {
        let tag = tag_name.to_lowercase().trim().to_string();

        if tag.len() == 0 
            || tag.len() > 32 
            || self.added_tags.contains_key(&tag) {
            return false
        }

        if self.added_tags.len() >= 255 {
            return false
        }

        // ID should not be zero as it is an indicator of no match
        let id = self.added_tags.len() + 1;
        let id_i16 = id as i16;

        self.added_tags[&tag] = id_i16;

        // remember tag string: it will be returned in case of matching
        self.strings[id] = Some(tag);

        // add both lower...
        if !self.add_tag_internal(tag, id, id*2+0) {
            return false
        }
           
        // ...and upper case tag values
        if !self.add_tag_internal(tag.to_uppercase(), id, id*2+1) {
            return false
        }

        // allocate memory for attribute hashes for this tag
        self.attr_data[id] = vec![0; 256];

        // now add attribute names
        let names = attr_names.to_lowercase().split(",");

        for name in names {
            let att_name = name.trim().to_string();

            if att_name.len() == 0 {
                continue
            }

            // only add attribute if we have not got it added 
            // for same first char of the same tag:
            let first_ch = att_name.chars().nth(0).unwrap();

            if self.attr_data[id][first_ch as usize] > 0
                || self.attr_data[id][first_ch.to_uppercase().unwrap()] > 0 {
                continue
            }

            let attr_id = if self.added_attributes.contains_key(&att_name) {
                self.added_attributes[&att_name]
            } else {
                let new_id = self.added_attributes.len() + 1;
                self.added_attributes[&att_name] = new_id as i16;
                self.attrs[new_id] = Some(att_name);

                new_id as i16
            };

            // add both lower...
            self.add_attribute(att_name, id_i16, attr_id*2 + 0);

            // ... and upper case tag values
            self.add_attribute(att_name.to_uppercase(), id_i16, attr_id*2 + 1);
        }

        true
    }

    fn add_attribute(&mut self, attr: String, id: i16, attr_id: i16) {
        if attr.len() == 0 {
            return
        }

        let b = attr.chars().nth(0).unwrap() as u8;

        self.attributes[attr_id as usize] = attr.as_bytes();
        self.attr_data[id as usize][b as usize] = attr_id as u8;
    }

    fn add_tag_internal(&mut self, tag: String, tag_id: usize, data_id: usize) -> bool {
        if tag.len() == 0 {
            return false
        }

        self.tag_data[data_id].push(tag.as_bytes());

        let tag_chars = tag.chars();
        let first_ch = tag_chars.nth(1).unwrap();

        if tag.len() == 1 {
            let id = -1i16 * (data_id as i16);

            // ok just one char, in which case we will mark possible second char that can be
            // '>', ' ' and other whitespace
            // we will use negative ID to hint that this is single char hit
            if !self.set_hash(first_ch, ' ', id) {
                return false
            }

            if !self.set_hash(first_ch, '\t', id) {
                return false
            }

            if !self.set_hash(first_ch, '\r', id) {
                return false
            }

            if !self.set_hash(first_ch, '\n', id) {
                return false
            }

            if !self.set_hash(first_ch, '>', id) {
                return false
            }

        } else {
            if !self.set_hash(first_ch, tag_chars.nth(1).unwrap(), data_id as i16) {
                return false
            }
        }

        true
    }

    fn set_hash(&mut self, ch1: char, ch2: char, id: i16) -> bool {
        let i = ch1 as usize;
        let j = ch2 as usize;

        //check if already exists
        if self.chars[i][j] != 0 {
            return false
        }

        self.chars[i][j] = id;
        true
    }
}