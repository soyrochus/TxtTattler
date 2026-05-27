//! Pure helpers with no I/O: text cleanup, chunking, cache keys, MP3 stitching.
//! Easy to unit-test because nothing here touches the network or disk.

use sha2::{Digest, Sha256};

use crate::domain::entities::{CacheKey, TtsOptions};
use crate::domain::ports::TextProcessor;

/// Bump this whenever the text-cleanup logic changes so old cache entries are
/// transparently invalidated (a stale clean → wrong audio bug, avoided).
pub const CACHE_VERSION: &str = "v1";

/// OpenAI's TTS endpoint rejects inputs longer than ~4096 characters, so we
/// keep a little headroom and split anything bigger.
pub const MAX_CHUNK_CHARS: usize = 4000;

/// A content-addressed cache key: same text + same options + same cleanup
/// version ⇒ same key ⇒ same cached MP3. Filename plays no part on purpose.
pub fn compute_cache_key(text: &str, options: &TtsOptions) -> CacheKey {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(b"\x00");
    hasher.update(options.voice.as_str().as_bytes());
    hasher.update(b"\x00");
    hasher.update(options.model.as_str().as_bytes());
    hasher.update(b"\x00");
    // Hash the raw bits so 1.0 and 1.0 always match regardless of formatting.
    hasher.update(options.speed.to_bits().to_le_bytes());
    hasher.update(b"\x00");
    hasher.update(CACHE_VERSION.as_bytes());
    CacheKey(hex::encode(hasher.finalize()))
}

/// Normalize whitespace: unify line endings, collapse 3+ blank lines into one,
/// and squash runs of spaces/tabs. Keeps paragraph structure for nicer pacing.
pub fn clean_text(text: &str) -> String {
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");

    // Collapse runs of newlines down to at most two (one blank line).
    let mut collapsed = String::with_capacity(unified.len());
    let mut newline_run = 0usize;
    for ch in unified.chars() {
        if ch == '\n' {
            newline_run += 1;
            if newline_run <= 2 {
                collapsed.push('\n');
            }
        } else {
            newline_run = 0;
            collapsed.push(ch);
        }
    }

    // Squash horizontal whitespace within each line and trim trailing spaces.
    collapsed
        .lines()
        .map(|line| {
            let mut out = String::with_capacity(line.len());
            let mut prev_space = false;
            for ch in line.chars() {
                if ch == ' ' || ch == '\t' {
                    if !prev_space {
                        out.push(' ');
                    }
                    prev_space = true;
                } else {
                    out.push(ch);
                    prev_space = false;
                }
            }
            out.trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Split text into chunks of at most `max_chars` *characters* (not bytes),
/// preferring paragraph then sentence boundaries so the audio doesn't get cut
/// off mid-word. A pathologically long sentence is hard-split as a last resort.
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    debug_assert!(max_chars > 0);

    if text.chars().count() <= max_chars {
        let trimmed = text.trim();
        return if trimmed.is_empty() {
            Vec::new()
        } else {
            vec![trimmed.to_string()]
        };
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in text.split("\n\n") {
        for sentence in split_sentences(paragraph) {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // A single oversized sentence can't share a chunk with anything.
            if char_len(sentence) > max_chars {
                flush(&mut current, &mut chunks);
                for piece in hard_split(sentence, max_chars) {
                    chunks.push(piece);
                }
                continue;
            }

            let separator = if current.is_empty() { 0 } else { 1 };
            if char_len(&current) + separator + char_len(sentence) > max_chars {
                flush(&mut current, &mut chunks);
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(sentence);
        }
    }

    flush(&mut current, &mut chunks);
    chunks
}

fn flush(current: &mut String, chunks: &mut Vec<String>) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        chunks.push(trimmed.to_string());
    }
    current.clear();
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Naive sentence splitter on `.`, `!`, `?` followed by whitespace. Good enough
/// for pacing; we never lose characters because the delimiter stays attached.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];
        current.push(ch);
        let is_terminator = ch == '.' || ch == '!' || ch == '?';
        let next_is_space = chars.get(i + 1).is_some_and(|c| c.is_whitespace());
        if is_terminator && next_is_space {
            sentences.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        sentences.push(current);
    }
    sentences
}

/// Last-resort splitter for a single sentence longer than the limit.
fn hard_split(text: &str, max_chars: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<char>>()
        .chunks(max_chars)
        .map(|window| window.iter().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Stitch per-chunk MP3 segments into one stream. MP3 frames are
/// self-contained, so naive concatenation yields a perfectly playable file.
pub fn concat_mp3(segments: &[Vec<u8>]) -> Vec<u8> {
    segments.iter().flatten().copied().collect()
}

/// The out-of-the-box text processor: just `clean_text`. Wrap or replace it to
/// add markdown stripping, abbreviation expansion, etc.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultTextProcessor;

impl TextProcessor for DefaultTextProcessor {
    fn process(&self, text: &str) -> String {
        clean_text(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{TtsModel, Voice};

    #[test]
    fn clean_text_collapses_whitespace_and_blank_lines() {
        let messy = "Hello    world\r\n\n\n\nGoodbye\t\tnow   ";
        assert_eq!(clean_text(messy), "Hello world\n\nGoodbye now");
    }

    #[test]
    fn short_text_stays_in_one_chunk() {
        let chunks = chunk_text("Just a little secret.", MAX_CHUNK_CHARS);
        assert_eq!(chunks, vec!["Just a little secret.".to_string()]);
    }

    #[test]
    fn empty_text_produces_no_chunks() {
        assert!(chunk_text("   \n\n  ", MAX_CHUNK_CHARS).is_empty());
    }

    #[test]
    fn long_text_is_chunked_within_the_limit() {
        let sentence = "The quick brown fox jumps over the lazy dog. ";
        let text = sentence.repeat(50); // well over the small limit below
        let chunks = chunk_text(&text, 100);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.chars().count() <= 100, "chunk too long: {chunk:?}");
        }
        // No content should be silently dropped.
        let rejoined: String = chunks.join(" ").split_whitespace().collect();
        let original: String = text.split_whitespace().collect();
        assert_eq!(rejoined, original);
    }

    #[test]
    fn oversized_sentence_is_hard_split() {
        let giant = "x".repeat(250);
        let chunks = chunk_text(&giant, 100);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|c| c.chars().count() <= 100));
    }

    #[test]
    fn cache_key_changes_with_options() {
        let base = TtsOptions::default();
        let key_a = compute_cache_key("hello", &base);
        let key_b = compute_cache_key("hello", &base);
        assert_eq!(key_a, key_b, "identical inputs must match");

        let other = TtsOptions {
            voice: Voice::Onyx,
            model: TtsModel::Tts1Hd,
            speed: 2.0,
        };
        assert_ne!(key_a, compute_cache_key("hello", &other));
        assert_ne!(key_a, compute_cache_key("HELLO", &base));
    }

    #[test]
    fn concat_mp3_joins_segments_in_order() {
        let joined = concat_mp3(&[vec![1, 2], vec![3], vec![4, 5]]);
        assert_eq!(joined, vec![1, 2, 3, 4, 5]);
    }
}
