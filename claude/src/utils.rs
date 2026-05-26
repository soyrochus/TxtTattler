use sha2::{Digest, Sha256};

use crate::domain::{entities::CacheKey, ports::TextProcessor};

/// Compute a stable cache key for the given text + TTS parameters.
pub fn compute_cache_key(text: &str, voice: &str, model: &str, speed: f32) -> CacheKey {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(b"|");
    hasher.update(voice.as_bytes());
    hasher.update(b"|");
    hasher.update(model.as_bytes());
    hasher.update(b"|");
    // Use the raw bits of the f32 for deterministic hashing
    hasher.update(&speed.to_bits().to_le_bytes());
    hasher.update(b"|v1");
    let result = hasher.finalize();
    CacheKey(hex::encode(result))
}

/// Split text into chunks of at most `max_chars` characters.
///
/// Tries to split at paragraph boundaries first (`\n\n`), then sentence
/// boundaries (`. `, `! `, `? `).  If a single paragraph/sentence is longer
/// than `max_chars` it is split at the character boundary.
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    // Fast path: fits in one chunk
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    // Split by paragraphs
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    for paragraph in paragraphs {
        // If adding this paragraph would overflow, try sentence splitting
        if !current.is_empty() && current.len() + paragraph.len() + 2 > max_chars {
            // Try to split the current paragraph into sentences
            let sentences = split_sentences(paragraph);
            for sentence in sentences {
                if current.len() + sentence.len() + 1 > max_chars {
                    if !current.is_empty() {
                        chunks.push(current.trim().to_string());
                        current = String::new();
                    }
                    // If a single sentence is still too long, hard-split it
                    if sentence.len() > max_chars {
                        for hard_chunk in hard_split(&sentence, max_chars) {
                            chunks.push(hard_chunk);
                        }
                    } else {
                        current.push_str(&sentence);
                    }
                } else {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(&sentence);
                }
            }
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(paragraph);
        }

        // Flush if we've exceeded the limit
        if current.len() >= max_chars {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

/// Split text at sentence boundaries (`. `, `! `, `? `).
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        current.push(chars[i]);
        if (chars[i] == '.' || chars[i] == '!' || chars[i] == '?')
            && i + 1 < chars.len()
            && chars[i + 1] == ' '
        {
            sentences.push(current.trim().to_string());
            current = String::new();
            i += 2; // skip the space
            continue;
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    sentences
}

/// Hard-split text into chunks of at most `max_chars` characters.
fn hard_split(text: &str, max_chars: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<char>>()
        .chunks(max_chars)
        .map(|c| c.iter().collect())
        .collect()
}

/// Concatenate multiple MP3 byte arrays. MP3 frames are independent, so
/// simple concatenation produces a valid (if slightly larger) stream.
pub fn concat_mp3_chunks(chunks: Vec<Vec<u8>>) -> Vec<u8> {
    chunks.into_iter().flatten().collect()
}

/// Normalize whitespace and collapse multiple blank lines into one.
pub fn clean_text(text: &str) -> String {
    // Normalize line endings
    let text = text.replace("\r\n", "\n").replace('\r', "\n");

    // Collapse runs of 3+ newlines down to 2 (one blank line)
    let mut result = String::with_capacity(text.len());
    let mut newline_count = 0usize;

    for ch in text.chars() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(ch);
            }
        } else {
            newline_count = 0;
            result.push(ch);
        }
    }

    // Collapse multiple spaces on a single line to one space
    result
        .lines()
        .map(|line| {
            let mut out = String::new();
            let mut last_was_space = false;
            for ch in line.chars() {
                if ch == ' ' || ch == '\t' {
                    if !last_was_space {
                        out.push(' ');
                    }
                    last_was_space = true;
                } else {
                    out.push(ch);
                    last_was_space = false;
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Default text processor — cleans/normalizes text before synthesis.
pub struct DefaultTextProcessor;

impl TextProcessor for DefaultTextProcessor {
    fn process(&self, text: &str) -> String {
        clean_text(text)
    }
}
