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
    /// themselves AND between them will be set to html variable, otherwise it will be empty
    /// but you can always set it later
    keep_scripts: bool,

    /// If true (and either keep_comments or keep_scripts is true), then html will be set
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
    html_bytes: Option<Box<[u8]>>, 

    /// Current position pointing to byte in html_bytes
    current_position: u32,

    /// Length of bHTML -- it appears to be faster to use it than html_bytes.len()
    data_length: u32,

    /// Whitespace lookup table - false is not whitespace, otherwise it is
    whitespace: [bool; 256],

    /// Entities manager
    entities: HtmlEntities,
}

impl HtmlParser {
    pub fn new() -> HtmlParser {
        let mut heuristics = HtmlHeuristics::new();
        let text = DynamicString::new("".to_string());
        let chunk = HtmlChunk{};
        let tag_parser = TagParser{};
        let encoding = "utf8".to_string();
        let html_bytes = None;
        let entities = HtmlEntities{};
        let mut whitespace = [false; 256];

        HtmlParser::init_whitespaces(whitespace);
        HtmlParser::init_heuristics(&mut heuristics);

        let parser = HtmlParser{
            decode_mini_entities: false,
            keep_raw_html: false,
            keep_comments: true,
            keep_scripts: true,
            extract_between_tags_only: true,
            mark_closed_tags_with_params_as_open: true,
            compress_whitespace_before_tag: true,
            heuristics: heuristics,
            text: text,
            chunk: chunk,
            tag_parser: tag_parser,
            encoding: encoding,
            html_bytes: html_bytes,
            current_position: 0,
            data_length: 0,
            entities: entities,
            whitespace: whitespace,
        };

        parser
    }

    /// sets flags of whitespace bytes to true
    fn init_whitespaces(mut whitespace: [bool; 256]) {
        whitespace[9] = true;
        whitespace[10] = true;
        whitespace[13] = true;
        whitespace[0x20] = true;
    }

    // init heuristics engine
    fn init_heuristics(heuristics: &mut HtmlHeuristics) {
        heuristics.add_tag("a", "href");
        heuristics.add_tag("b", "");
        heuristics.add_tag("p", "class");
        heuristics.add_tag("i", "");
        heuristics.add_tag("s", "");
        heuristics.add_tag("u", "");

        heuristics.add_tag("td", "align,valign,bgcolor,rowspan,colspan");
        heuristics.add_tag("table", "border,width,cellpadding");
        heuristics.add_tag("span", "");
        heuristics.add_tag("option", "");
        heuristics.add_tag("select", "");

        heuristics.add_tag("tr", "");
        heuristics.add_tag("div", "class,align");
        heuristics.add_tag("img", "src,width,height,title,alt");
        heuristics.add_tag("input", "");
        heuristics.add_tag("br", "");
        heuristics.add_tag("li", "");
        heuristics.add_tag("ul", "");
        heuristics.add_tag("ol", "");
        heuristics.add_tag("hr", "");
        heuristics.add_tag("h1", "");
        heuristics.add_tag("h2", "");
        heuristics.add_tag("h3", "");
        heuristics.add_tag("h4", "");
        heuristics.add_tag("h5", "");
        heuristics.add_tag("h6", "");
        heuristics.add_tag("font", "size,color");
        heuristics.add_tag("meta", "name,content,http-equiv");
        heuristics.add_tag("base", "href");
        
        // these are pretty rare
        heuristics.add_tag("script", "");
        heuristics.add_tag("style", "");
        heuristics.add_tag("html", "");
        heuristics.add_tag("body", "");
    }
}