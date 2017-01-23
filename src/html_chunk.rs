use std::collections::hash_map::HashMap;
use encoding::{Encoding, EncodingRef, EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use encoding::all::ASCII;

/// Type of parsed HTML chunk (token), each non-null returned chunk from HTMLparser will have oType set to 
/// one of these values
pub enum ChunkType {
    /// Text data from HTML
    Text = 0,

    /// Open tag, possibly with attributes
    OpenTag = 1,

    /// Closed tag (it may still have attributes)
    CloseTag = 2,

    /// Comment tag (<!-- -->)depending on HtmlParser boolean flags you may have:
    /// a) nothing to html variable - for faster performance, call set_raw_html function in parser
    /// b) data BETWEEN tags (but not including comment tags themselves) - DEFAULT
    /// c) complete RAW HTML representing data between tags and tags themselves (same as you get in a) when
    /// you call set_raw_html function)
    /// 
    /// Note: this can also be CDATA part of XML document - see tag value to determine if its proper comment
    /// or CDATA or (in the future) something else
    Comment = 3,

    /// Script tag (<!-- -->) depending on HtmlParser boolean flags
    /// a) nothing to html variable - for faster performance, call set_raw_html function in parser
    /// b) data BETWEEN tags (but not including comment tags themselves) - DEFAULT
    /// c) complete RAW HTML representing data between tags and tags themselves (same as you get in a) when
    /// you call set_raw_html function)
    Script = 4,
}

/// Maximum number of parameters in a 
/// tag - should be high enough to fit most sensible cases
const MAX_PARAMS: usize = 256;

/// Parsed HTML token that is either text, comment, script, 
/// open or closed tag as indicated by the type variable.
pub struct HtmlChunk {
    /// Chunk type showing whether its text, open or close tag, comments or script.
    /// WARNING: if type is comments or script then you have to manually call Finalise(); method
    /// in order to have actual text of comments/scripts in oHTML variable
    chunk_type: ChunkType,

    /// If true then tag params will be kept in a hash rather than in a fixed size arrays. 
    /// This will be slow down parsing, but make it easier to use.
    hash_mode: bool,

    /// For TAGS: it stores raw HTML that was parsed to generate thus chunk will be here UNLESS
    /// HTMLparser was configured not to store it there as it can improve performance
    /// <p>
    /// For TEXT or COMMENTS: actual text or comments - you MUST call Finalise(); first.
    /// </p>
    html: String,

    /// Offset in html_data data array at which this chunk starts
    chunk_offset: usize,

    /// Length of the chunk in bHTML data array
    chunk_length: usize,

    /// If its open/close tag type then this is where lowercased Tag will be kept
    tag: String,

    /// If true then it must be closed tag
    closure: bool,

    /// If true then it must be closed tag and closure sign / was at the END of tag, ie this is a SOLO
    /// tag 
    end_closure: bool,

    /// If true then it must be comments tag
    comments: bool,

    /// True if entities were present (and transformed) in the original HTML
    entities: bool,

    /// Set to true if &lt; entity (tag start) was found 
    lt_entity: bool,

    /// Hashtable with tag parameters: keys are param names and values are param values.
    /// ONLY used if hash_mode is set to true.
    params: Option<HashMap<String, String>>,

    /// Number of parameters and values stored in param_names array, OR in params hashtable if
    /// hash_mode is true
    params_count: usize,

    /// Param names will be stored here - actual number is in params_count.
    /// ONLY used if hash_mode is set to false.
    param_names: Vec<String>,

    /// Param values will be stored here - actual number is in params_count.
    /// ONLY used if hash_mode is set to false.
    param_values: Vec<String>,

    /// Character used to quote param's value: it is taken actually from parsed HTML
    param_chars: Vec<u8>,

    /// Encoder to be used for conversion of binary data into strings, ASCII is used by default,
    /// but it can be changed if top level user of the parser detects that encoding was different
    enc: EncodingRef,
}

impl HtmlChunk {
    pub fn new(hash_mode: bool) -> HtmlChunk {
        let params_hash: Option<HashMap<String, String>> = if hash_mode {
            Some(HashMap::new())
        } else {
            None
        };
        
        HtmlChunk {
            hash_mode: hash_mode,
            chunk_type: ChunkType::Text,
            html: String::from(""),
            chunk_offset: 0,
            chunk_length: 0,
            tag: String::from(""),
            closure: false,
            end_closure: false,
            comments: false,
            entities: false,
            lt_entity: false,
            params: params_hash,
            params_count: 0,
            param_names: Vec::new(),
            param_chars: Vec::new(),
            param_values: Vec::new(),
            enc: encoding_from_whatwg_label("ascii").unwrap(),
        }
    }

    /// This function will convert parameters stored in param_names/param_values arrays into params hash
    /// Useful if generally parsing is done when hash_mode is true. Hash operations are not the fastest, so
    /// its best not to use this function.
    pub fn convert_params_to_hash(&mut self) {
        if let Some(ref mut hash) = self.params {
            hash.clear();
        } else {
            self.params = Some(HashMap::new());
        }

        if let Some(ref mut h) = self.params {
            for i in 0..self.params_count {
                h.insert(self.param_names[i].clone(), self.param_values[i].clone());
            }
        }
    }

    /// Sets encoding to be used for conversion of binary data into string
    pub fn set_encoding(&mut self, encoding: EncodingRef) {
        self.enc = encoding;
    }

    /// Clears chunk preparing it for 
    pub fn clear(&mut self) {
        self.tag.clear();
        self.html.clear();

        self.lt_entity = false;
        self.entities = false;
        self.comments = false;
        self.closure = false;
        self.end_closure = false;

        self.params_count = 0;

        if self.hash_mode {
            if let Some(ref mut hash) = self.params {
                hash.clear();
            }
        }
    }

    /// Generates HTML based on current chunk's data 
    /// Note: this is not a high performance method and if you want ORIGINAL HTML that was parsed to create
    /// this chunk then use relevant HtmlParser method to obtain such HTML then you should use
    /// function of parser: set_raw_html
    pub fn generate_html(&self) -> String {
        let mut new_html: String = String::from("");

        match self.chunk_type {
			// matched open tag, ie <a href="">
            ChunkType::OpenTag => {
                new_html = new_html + "<" + &self.tag;

                if self.params_count > 0 {
                    new_html = new_html + " " + &self.generate_params_html();
                }

                new_html += ">";
            },
            // matched close tag, ie </a>
            ChunkType::CloseTag => {
                if self.params_count > 0 || self.end_closure {
                    new_html = new_html + "<" + &self.tag;

                    if self.params_count > 0 {
                        new_html = new_html + " " + &self.generate_params_html();
                    }

                    new_html += "/>";
                } else {
                    new_html = new_html + "</" + &self.tag + ">";
                }
            },
            ChunkType::Script => {
                if self.html.len() == 0 {
                    new_html = String::from("<script>n/a</script>");
                } else {
                    new_html = self.html.clone();
                }
            },
            ChunkType::Comment => {
                // note: we might have CDATA here that we treat as comments
                if self.tag == "!--" {
                    if self.html.len() == 0 {
                        new_html = String::from("<!-- n/a -->");
                    } else {
                        new_html = String::from("<!--") + &self.html + "-->";
                    }
                } else {
                    // ref: http://www.w3schools.com/xml/xml_cdata.asp
                    if self.tag == "![CDATA[" {
                        if self.html.len() == 0 {
                            new_html = String::from("<![CDATA[ n/a \n]]>");
                        } else {
                            new_html = String::from("<![CDATA[") + &self.html + "]]>";
                        }
                    }
                }
            },
            // matched normal text
            ChunkType::Text => {
                new_html = self.html.clone();
            }
        }

        new_html
    }

    fn generate_params_html(&self) -> String {
        let mut new_html = String::from("");

        if self.hash_mode {
            if let Some(ref params) = self.params {
                if params.len() > 0 {
                    for (k, v) in params.iter() {
                        if new_html.len() > 0 {
                            new_html += " ";
                        }

                        // FIXIT: this is really not correct as we do not use same char used
                        new_html = new_html + &self.generate_param_html(k, v, '\'');
                    }
                }
            }
        } else {
            // this is alternative method of getting params -- it may look less convinient
            // but it saves a LOT of CPU ticks while parsing. It makes sense when you only need
            // params for a few
            if self.params_count > 0 {
                for i in 0..self.params_count {
                    if new_html.len() > 0 {
                        new_html += " ";
                    }

                    new_html += &self.generate_param_html(
                        &self.param_names[i], 
                        &self.param_values[i], 
                        self.param_chars[i] as char);
                }
            }
        }

        new_html
    }

    /// Returns value of a parameter
    fn get_param_value(&self, name: &String) -> String {
        if self.hash_mode {
            if let Some(ref params) = self.params {
                if let Some(val) = params.get(name) {
                    return val.clone();
                }
            }
        } else {
            for i in 0..self.params_count {
                if self.param_values[i] == *name {
                    return self.param_values[i].clone();
                }
            }
        }

        return String::from("");
    }

    fn generate_param_html(&self, name: &String, val: &String, ch: char) -> String {
        if val.len() > 0 {
            if val.len() > 20 {
                return String::from(name.clone() + "=" + &ch.to_string() + &self.make_safe_param_value(&val, ch) + &ch.to_string());
            }

            for val_ch in val.chars() {
                match val_ch {
                    ' ' | '\t' | '\'' | 
                    '\"' | '\n' | '\r' => {
                        return String::from(name.clone() + "='" + &self.make_safe_param_value(&val, '\'') + "'");
                    },
                    _ => {}
                }
            }

            return String::from(name.clone() + "=" + &val);
        } 
            
        name.clone()
    }

    /// Makes parameter value safe to be used in 
    /// param - this will check for any conflicting quote chars,
    /// but not full entity-encoding
    fn make_safe_param_value(&self, line: &String, quote_ch: char) -> String {
        // we speculatievly expect that in most cases 
        // we don't actually need to entity-encode string
        for i in 0..line.len() {
            if line.chars().nth(i).unwrap() != quote_ch {
                continue;
            }

            // have to restart here
            let mut new_s: String = line.chars().take(i).collect();

            for j in i..line.len() {
                let ch = line.chars().nth(j).unwrap();

                if ch == quote_ch {
                    new_s = new_s + "&#" + &(ch as usize).to_string() + ";";
                } else {
                    new_s = new_s + &ch.to_string();
                }
            }

            return new_s;
        }

        line.clone()
    }
}