use encoding::{Encoding, EncodingRef, EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use encoding::all::ASCII;

const TEXT_CAPACITY: usize = 1024*256-1;

/// Class for fast dynamic string building - it is faster than StringBuilder
pub struct DynamicString {
    /// Finalised text will be available in this string
    text: String,

    buffer: [u8; TEXT_CAPACITY+1],
    buffer_pos: usize,
    length: usize,

    enc: EncodingRef,
}

impl DynamicString {
    pub fn new(s: String) -> DynamicString {
        DynamicString {
            length: s.len(),
            text: s,
            enc: encoding_from_whatwg_label("utf8").unwrap(),
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
    pub fn set_encoding(&mut self, encoding: EncodingRef) {
        self.enc = encoding;
    }

    pub fn append(&mut self, ch: char) {
        if ch as u8 <= 127 {
            self.buffer[self.buffer_pos] = ch as u8;
            self.buffer_pos += 1;
        } else {
            // unicode character - this is really bad way of doing it, but 
            // it seems to be called almost never
            let mut bytes = Vec::new();
            self.enc.encode_to(&ch.to_string(), EncoderTrap::Ignore, &mut bytes);

            // 16/09/07 Possible bug reported by Martin Bächtold: 
            // test case: 
            // <meta http-equiv="Content-Category" content="text/html; charset=windows-1251">
            // &#1329;&#1378;&#1400;&#1406;&#1397;&#1377;&#1398; &#1341;&#1377;&#1401;&#1377;&#1407;&#1400;&#1410;&#1408;

            // the problem is that some unicode chars might not be mapped to bytes by specified encoding
            // in the HTML itself, this means we will get single byte ? - this will look like failed conversion
            // Not good situation that we need to deal with :(
            if bytes.len() == 1 || bytes[0] == '?' as u8 {
                // TODO: 
                for b in bytes {
                    self.buffer[self.buffer_pos] = b;
                    self.buffer_pos += 1;
                }
            } else {
                for b in bytes {
                    self.buffer[self.buffer_pos] = b;
                    self.buffer_pos += 1;
                }
            }
        }
    }

    /// Creates string from buffer using set encoder
    fn set_to_string(&mut self) -> &String {
        if self.buffer_pos > 0 {
            if self.text.len() == 0 {
                self.text = self.enc.decode(&self.buffer[0..self.buffer_pos], DecoderTrap::Ignore).unwrap();
            } else {
                self.text += &self.enc.decode(&self.buffer[0..self.buffer_pos], DecoderTrap::Ignore).unwrap();
            }

            self.length += self.buffer_pos;
            self.buffer_pos = 0;
        }

        &self.text
    }

    /// Creates string from buffer using default encoder
    fn set_to_string_ascii(&mut self) -> &String {
        if self.buffer_pos > 0 {
            if self.text.len() == 0 {
                self.text = ASCII.decode(&self.buffer[0..self.buffer_pos], DecoderTrap::Ignore).unwrap();
            } else {
                self.text += &ASCII.decode(&self.buffer[0..self.buffer_pos], DecoderTrap::Ignore).unwrap();
            }

            self.length += self.buffer_pos;
            self.buffer_pos = 0;
        }

        &self.text
    }
}