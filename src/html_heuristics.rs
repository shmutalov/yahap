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

    pub fn add_tag(&mut self, tag: &str, attributes: &str) -> bool {
        true
    }

    /// Returns String of i and j combination
    pub fn get_two_char_string(i: u8, j: u8) -> String {
        ALL_TWO_CHARS[(i as usize)*256 + (j as usize)].clone()
    }

    
}