#[macro_use]
extern crate lazy_static;
extern crate encoding;

mod html_heuristics;
mod dynamic_string;
mod html_chunk;
mod tag_parser;
mod html_entities;

pub mod html_parser;