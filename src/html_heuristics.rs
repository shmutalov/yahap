use std::collections::hash_map::HashMap;

/// Maximum number of strings allowed to be set (all lower-cased)
const MAX_STRINGS: usize = 1024;

/// Maximum number of chars to be taken into account
const MAX_CHARS: usize = 255;

/// This class will control HTML tag heuristics that will allow faster matching of tags
/// to avoid long cycles as well as creation of same strings over and over again.
/// 
/// This is effectively a fancy hash lookup table with attributes being hashed in context of tag
pub struct HtmlHeuristics {
    /// Array in which we will keep char hints to quickly match	ID (if non-zero) of tag
    chars: [[i16; 256]; 256],

    /// Strings used, once matched they will be returned to avoid creation of a brand new string
    /// and all associated costs with it
    strings: [Option<String>; MAX_STRINGS],

    /// Binary data represending tag strings is here: case sensitive: lower case for even even value, and odd for each odd
    /// for the same string
    tag_data: [Option<Box<[u8]>>; MAX_STRINGS*2],

    /// List of added tags to avoid dups
    added_tags: HashMap<String, i16>,

    /// Hash that will contain single char mapping hash
    attributes: [Option<Box<[u8]>>; MAX_STRINGS*2],

    /// Binary data represending attribute strings is here: case sensitive: lower case for even even value, and odd for each odd
    /// for the same string
    attr_data: [Option<Box<[u8]>>; MAX_STRINGS*2],

    /// List of added attributes to avoid dups
    added_attributes: HashMap<String, i16>,

    attrs: [Option<String>; MAX_STRINGS],

    /// This array will contain all double char strings 
    all_two_char_strings: [[Option<String>; MAX_CHARS+1]; MAX_CHARS+1],
}

impl HtmlHeuristics {
    pub fn new() -> HtmlHeuristics {
        let chars = [[0; 256]; 256];
        let strings = [None; MAX_STRINGS];
        let tag_data = [None; MAX_STRINGS*2];
        let added_tags: HashMap<String, i16> = HashMap::new();
        let attributes = [None; MAX_STRINGS*2];
        let attr_data = [None; MAX_STRINGS*2];
        let added_attributes: HashMap<String, i16> = HashMap::new();
        let attrs = [None; MAX_STRINGS];
        let mut all_two_char_strings = [[None; MAX_CHARS+1]; MAX_CHARS+1];

        HtmlHeuristics::init_all_two_char_strings(all_two_char_strings);

        let heuristics = HtmlHeuristics {
            chars: chars,
            strings: strings,
            tag_data: tag_data,
            added_tags: added_tags,
            attributes: attributes,
            attr_data: attr_data,
            added_attributes: added_attributes,
            attrs: attrs,
            all_two_char_strings: all_two_char_strings,
        };

        heuristics
    }

    fn init_all_two_char_strings(mut arr: [[Option<String>; MAX_CHARS+1]; MAX_CHARS+1]) {
        // we will create all possible strings for two bytes combinations - this will allow
        // to cater for all two char combinations at cost of mere 256kb of RAM per instance
        for i in 0 .. arr.len() {
            let ch1 = std::char::from_digit(i >> 8);
            let ch2 = std::char::from_digit(i & 0xff);

            arr[b1][b2] = Some(ch1 + ch2);
        }
    }

    pub fn add_tag(&mut self, tag: &str, attributes: &str) -> bool {
        true
    }
}