//! Content-addressable MP3 cache for TxtTattler.
//!
//! Cache key is a SHA-256 of:
//!   - cleaned document text
//!   + voice
//!   + model
//!   + speed (as string)
//!   + a CACHE_VERSION constant (bump this when text processing or concat logic changes)
//!
//! This guarantees that any change in output will correctly invalidate old cache entries.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Bump this whenever the caching behavior or final MP3 format changes in a
/// way that would produce different audio for the same inputs.
pub const CACHE_VERSION: &str = "v1";

/// The tattler's memory — a polite little gossip archive on disk.
pub struct Mp3Cache {
    root: PathBuf,
}

impl Mp3Cache {
    /// Create a cache rooted at the platform default (or an overridden path).
    pub fn new(override_dir: Option<PathBuf>) -> Result<Self> {
        let root = if let Some(dir) = override_dir {
            dir
        } else {
            // Cross-platform sensible default
            let base = dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("txttattler");
            base
        };

        fs::create_dir_all(&root)
            .with_context(|| format!("Failed to create MP3 cache directory at {}", root.display()))?;

        Ok(Self { root })
    }

    /// Compute a stable, content-based cache filename (hex sha256).
    pub fn cache_key(
        &self,
        text: &str,
        voice: &str,
        model: &str,
        speed: f32,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.update(b"|"); // unambiguous separator
        hasher.update(voice.as_bytes());
        hasher.update(b"|");
        hasher.update(model.as_bytes());
        hasher.update(b"|");
        hasher.update(speed.to_string().as_bytes());
        hasher.update(b"|");
        hasher.update(CACHE_VERSION.as_bytes());

        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    /// Returns the full path we would use for this key (whether it exists or not).
    pub fn path_for_key(&self, key: &str) -> PathBuf {
        self.root.join(format!("{}.mp3", key))
    }

    /// Attempt to read a cached MP3. Returns the path on hit.
    pub fn get(&self, key: &str) -> Option<PathBuf> {
        let path = self.path_for_key(key);
        if path.exists() {
            // Basic sanity: non-empty file
            if let Ok(meta) = fs::metadata(&path) {
                if meta.len() > 100 {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Write (or overwrite) the cached MP3 for this key.
    /// We write atomically by using a temp file + rename.
    pub fn put(&self, key: &str, data: &[u8]) -> Result<PathBuf> {
        let final_path = self.path_for_key(key);
        let temp_path = final_path.with_extension("mp3.tmp");

        fs::write(&temp_path, data)
            .with_context(|| format!("Failed to write temp cache file at {}", temp_path.display()))?;

        fs::rename(&temp_path, &final_path)
            .with_context(|| format!("Failed to move cache file into place at {}", final_path.display()))?;

        Ok(final_path)
    }

    /// Remove a cached entry (used by --refresh).
    pub fn remove(&self, key: &str) -> Result<()> {
        let path = self.path_for_key(key);
        if path.exists() {
            fs::remove_file(&path).ok();
        }
        Ok(())
    }

    /// Human-friendly location for messages.
    #[allow(dead_code)]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
