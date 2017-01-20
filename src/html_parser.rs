use html_heuristics::HtmlHeuristics;
use dynamic_string::DynamicString;
use html_chunk::HtmlChunk;
use tag_parser::TagParser;
use html_entities::HtmlEntities;

/// Allows to parse HTML by splitting it into small token (HTMLchunks) such as tags, text, comments etc.
/// 
/// Do NOT create multiple instances of this class - REUSE single instance
/// Do NOT call same instance from multiple threads - it is NOT thread safe
pub struct HtmlParser {

    /// If false (default) then mini entity set (&nbsp;) will be decoded, but not all of them
    decode_mini_entities: bool,

    /// If true (default: false) then parsed tag chunks will contain raw HTML, 
    /// otherwise only comments will have it set
    /// 
    /// Performance hint: keep it as false, you can always get to original HTML as each chunk contains
    /// offset from which parsing started and finished, thus allowing to set exact HTML that was parsed
    keep_raw_html: bool,

    /// If true (default) then HTML for comments tags 
    /// themselves AND between them will be set to oHTML variable, otherwise it will be empty
    /// but you can always set it later 
    keep_comments: bool,

    /// If true (default: false) then HTML for script tags 
    /// themselves AND between them will be set to oHTML variable, otherwise it will be empty
    /// but you can always set it later
    keep_scripts: bool,

    /// If true (and either keep_comments or keep_scripts is true), then oHTML will be set
    /// to data BETWEEN tags excluding those tags themselves, as otherwise FULL HTML will be set, ie:
    /// '<!-- comments -->' but if this is set to true then only ' comments ' will be returned
    extract_between_tags_only: bool,

    /// Long winded name... by default if tag is closed BUT it has got parameters then we will consider it
    /// open tag, this is not right for proper XML parsing
    mark_closed_tags_with_params_as_open: bool,

    /// If true (default), then all whitespace before TAG starts will be compressed to single space char (32 or 0x20)
    /// this makes parser run a bit faster, if you need exact whitespace before tags then change this flag to FALSE
    compress_whitespace_before_tag: bool,

    /// Heuristics engine used by Tag Parser to quickly match known tags and attribute names, can be disabled
    /// or you can add more tags to it to fit your most likely cases, it is currently tuned for HTML
    heuristics: HtmlHeuristics,

    /// Internal -- dynamic string for text accumulation
    text: DynamicString,

    /// This chunk will be returned when it was parsed
    chunk: HtmlChunk,

    /// Tag parser object
    tag_parser: TagParser,

    /// Encoding used to convert binary data into string
    encoding: String,

    /// Byte array with HTML will be kept here
    html_bytes: Box<[u8]>, 

    /// Current position pointing to byte in html_bytes
    current_position: u32,

    /// Length of bHTML -- it appears to be faster to use it than html_bytes.len()
    data_length: u32,

    /// Whitespace lookup table - false is not whitespace, otherwise it is
    whitespace: [bool; 256],

    /// Entities manager
    entities: HtmlEntities,
}