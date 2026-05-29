use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::domain::entities::{ProcessedText, TtsOptions};
use crate::domain::ports::TextProcessor;

pub struct WhitespaceTextProcessor;

impl TextProcessor for WhitespaceTextProcessor {
    fn process(&self, text: &str) -> Result<ProcessedText> {
        Ok(ProcessedText {
            original_character_count: text.chars().count(),
            cleaned: clean_whitespace(text),
        })
    }
}

pub fn clean_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank_lines = 0usize;
    let mut pending_space = false;

    for ch in text.replace("\r\n", "\n").replace('\r', "\n").chars() {
        match ch {
            '\n' => {
                if pending_space && !out.ends_with('\n') {
                    out.push(' ');
                }
                pending_space = false;
                blank_lines += 1;
                if blank_lines <= 2 && !out.ends_with("\n\n") {
                    out.push('\n');
                }
            }
            ch if ch.is_whitespace() => {
                pending_space = true;
                blank_lines = 0;
            }
            _ => {
                if pending_space && !out.ends_with([' ', '\n']) {
                    out.push(' ');
                }
                pending_space = false;
                blank_lines = 0;
                out.push(ch);
            }
        }
    }

    out.trim().to_string()
}

pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let paragraphs = text.split("\n\n");
    let mut current = String::new();

    for paragraph in paragraphs {
        push_text_piece(paragraph, max_chars, &mut current, &mut chunks);
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

fn push_text_piece(piece: &str, max_chars: usize, current: &mut String, chunks: &mut Vec<String>) {
    if piece.chars().count() > max_chars {
        for sentence in split_long_piece(piece, max_chars) {
            append_chunk(&sentence, max_chars, current, chunks);
        }
    } else {
        append_chunk(piece, max_chars, current, chunks);
    }
}

fn append_chunk(piece: &str, max_chars: usize, current: &mut String, chunks: &mut Vec<String>) {
    let separator = if current.is_empty() { "" } else { "\n\n" };
    let projected_len = current.chars().count() + separator.chars().count() + piece.chars().count();
    if projected_len > max_chars && !current.is_empty() {
        chunks.push(current.trim().to_string());
        current.clear();
    }

    if !current.is_empty() {
        current.push_str("\n\n");
    }
    current.push_str(piece.trim());
}

fn split_long_piece(piece: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in piece.split_whitespace() {
        let separator = if current.is_empty() { "" } else { " " };
        let projected_len = current.chars().count() + separator.len() + word.chars().count();
        if projected_len > max_chars && !current.is_empty() {
            chunks.push(current);
            current = String::new();
        }

        if word.chars().count() > max_chars {
            for chunk in split_word(word, max_chars) {
                chunks.push(chunk);
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn split_word(word: &str, max_chars: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for ch in word.chars() {
        if current.chars().count() == max_chars {
            result.push(current);
            current = String::new();
        }
        current.push(ch);
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

pub fn cache_key(text: &str, options: &TtsOptions, version: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(version.as_bytes());
    hasher.update([0]);
    hasher.update(options.voice.to_string().as_bytes());
    hasher.update([0]);
    hasher.update(options.model.to_string().as_bytes());
    hasher.update([0]);
    hasher.update(format!("{:.2}", options.speed).as_bytes());
    hasher.update([0]);
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn concat_mp3_segments(segments: &[Vec<u8>]) -> Vec<u8> {
    let total = segments.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(total);
    for segment in segments {
        out.extend_from_slice(segment);
    }
    out
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let tmp = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ));
    fs::write(&tmp, bytes).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| {
        format!(
            "failed to move temporary file {} to {}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::{Model, Voice};

    use super::*;

    #[test]
    fn collapses_whitespace_without_flattening_paragraphs() {
        assert_eq!(
            clean_whitespace("hi   there\n\n\nfriend"),
            "hi there\n\nfriend"
        );
    }

    #[test]
    fn chunks_stay_under_limit() {
        let text = "alpha beta gamma delta epsilon zeta eta theta";
        let chunks = chunk_text(text, 12);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 12));
    }

    #[test]
    fn cache_key_changes_with_voice() {
        let a = cache_key(
            "hello",
            &TtsOptions {
                voice: Voice::Alloy,
                model: Model::Tts1,
                speed: 1.0,
            },
            "v1",
        );
        let b = cache_key(
            "hello",
            &TtsOptions {
                voice: Voice::Nova,
                model: Model::Tts1,
                speed: 1.0,
            },
            "v1",
        );
        assert_ne!(a, b);
    }
}
