use anyhow::Result;
use console::style;
use sha2::{Digest, Sha256};
use tracing_subscriber::EnvFilter;

use crate::domain::ports::TextProcessor;

pub fn banner() -> String {
    format!(
        "{}\n{}",
        style(r" _____     _ _____     _   _   _").magenta().bold(),
        style(r"|_   _|_ _| |_   _|_ _| |_| |_| | ___ _ __").magenta()
    )
}

pub fn init_tracing(verbose: bool) {
    let level = if verbose { "debug" } else { "warn" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

pub struct WhitespaceNormalizer;

impl TextProcessor for WhitespaceNormalizer {
    fn process(&self, text: String) -> Result<String> {
        Ok(normalize_whitespace(&text))
    }
}

pub fn normalize_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut blank_lines = 0usize;

    for line in input.lines() {
        let trimmed = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if trimmed.is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !out.ends_with('\n') {
                out.push('\n');
            }
            continue;
        }

        blank_lines = 0;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&trimmed);
    }

    out.trim().to_owned()
}

pub fn chunk_text(text: &str, limit: usize) -> Vec<String> {
    if text.chars().count() <= limit {
        return vec![text.to_owned()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in text.split("\n\n") {
        let candidate_len = current.chars().count() + paragraph.chars().count() + 2;
        if !current.is_empty() && candidate_len > limit {
            chunks.push(std::mem::take(&mut current));
        }

        if paragraph.chars().count() > limit {
            flush_long_paragraph(paragraph, limit, &mut chunks);
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(paragraph);
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current);
    }

    chunks
}

pub fn cache_key(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }

    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn flush_long_paragraph(paragraph: &str, limit: usize, chunks: &mut Vec<String>) {
    let mut current = String::new();

    for sentence in paragraph.split_inclusive(['.', '!', '?']) {
        let candidate_len = current.chars().count() + sentence.chars().count();
        if !current.is_empty() && candidate_len > limit {
            chunks.push(std::mem::take(&mut current));
        }

        if sentence.chars().count() > limit {
            for word in sentence.split_whitespace() {
                let extra = usize::from(!current.is_empty());
                if !current.is_empty()
                    && current.chars().count() + word.chars().count() + extra > limit
                {
                    chunks.push(std::mem::take(&mut current));
                }
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
        } else {
            current.push_str(sentence);
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_excess_whitespace() {
        assert_eq!(normalize_whitespace(" a   b \n\n\n c "), "a b\nc");
    }

    #[test]
    fn chunks_without_exceeding_limit() {
        let text = "word ".repeat(2000);
        let chunks = chunk_text(&text, 100);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 100));
    }

    #[test]
    fn cache_key_changes_with_parts() {
        assert_ne!(cache_key(&["same", "alloy"]), cache_key(&["same", "nova"]));
    }
}
