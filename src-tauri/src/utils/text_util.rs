use crate::global::{
    DOCUMENT_CHUNK_OVERLAP, DOCUMENT_CHUNK_SIZE, EN_TOKENIZER_PATH, MULTI_LANG_TOKENIZER_PATH,
};
use std::error::Error;
use std::path::Path;
use text_splitter::{ChunkCapacity, ChunkConfig, TextSplitter};
use tokenizers::Tokenizer;

pub fn collapse_newlines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_newline = false;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\r' {
            // Check if "\r\n" is present
            if let Some('\n') = chars.peek() {
                chars.next(); // Skip '\n'
            }

            if !last_was_newline {
                result.push('\n'); // Add a newline if the previous character was not a newline '\n'
                last_was_newline = true;
            }
        } else if ch == '\n' {
            if !last_was_newline {
                result.push('\n');
                last_was_newline = true;
            }
        } else {
            result.push(ch);
            last_was_newline = false;
        }
    }

    result
}

pub fn split_text(text: &str, tokenizer: &Tokenizer) -> Result<Vec<String>, Box<dyn Error>> {
    let chunk_capacity = ChunkCapacity::new(DOCUMENT_CHUNK_SIZE);
    match chunk_capacity.with_max(DOCUMENT_CHUNK_SIZE * 5) {
        Ok(_) => (),
        Err(e) => {
            println!("Error setting chunk capacity: {}", e);
        }
    }
    let config = ChunkConfig::new(DOCUMENT_CHUNK_SIZE)
        .with_sizer(&tokenizer)
        .with_overlap(DOCUMENT_CHUNK_OVERLAP)?;
    let splitter = TextSplitter::new(config);
    Ok(splitter.chunks(text).map(|x| x.to_string()).collect())
}
