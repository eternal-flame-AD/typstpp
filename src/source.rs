#![allow(dead_code)]
use std::collections::HashMap;

pub enum Chunk {
    Verbatim(String),
    Code(CodeChunk),
    Output(typstpp_backend::Output<String>),
    Message(String),
    Error(String),
    Graphics(GraphicsChunk),
}

pub struct CodeChunk {
    pub lang: String,
    pub options: HashMap<String, String>,
    pub code: String,
}

pub struct GraphicsChunk {
    pub data: Vec<u8>,
    pub ty: GraphicsType,
}

pub enum GraphicsType {
    Png,
}
