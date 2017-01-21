const TEXT_CAPACITY: usize = 1024*256-1;

/// Class for fast dynamic string building - it is faster than StringBuilder
pub struct DynamicString {
    /// Finalised text will be available in this string
    text: String,

    buffer: [u8; TEXT_CAPACITY+1],
    buffer_pos: usize,
    length: usize,

    encoding: String,
}

impl DynamicString {
    pub fn new(s: String) -> DynamicString {
        DynamicString {
            length: s.len(),
            text: s,
            encoding: "utf8".to_string(),
            buffer_pos: 0,
            buffer: [0; TEXT_CAPACITY+1],
        }
    }

    /// Resets object to zero length string
    pub fn clear(&mut self) {
        self.text = "".to_string();
        self.length = 0;
        self.buffer_pos = 0;
    }

    /// Sets encoding to be used for conversion of binary data into string
    pub fn set_encoding(&mut self, encoding: String) {
        self.encoding = encoding;
    }

    pub fn append(&mut self, ch: char) {
        if ch as u8 <= 127 {
            self.buffer[self.buffer_pos] = ch as u8;
            self.buffer_pos += 1;
        } else {
            
        }
    }
}