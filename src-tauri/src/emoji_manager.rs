//! Emoji Manager Module
//! Handles emoji usage tracking with LRU (Least Recently Used) semantics and disk persistence.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of recent emojis to track
const MAX_RECENT_EMOJIS: usize = 20;

/// Persistence filename
const EMOJI_HISTORY_FILE: &str = "emoji_history.json";

/// A single emoji usage entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmojiUsage {
    /// The emoji character
    pub char: String,
    /// Number of times used
    #[serde(default)]
    pub use_count: u32,
    /// Last used timestamp (Unix epoch millis)
    #[serde(default = "current_time_millis")]
    pub last_used: u64,
}

/// Persistent storage format wrapper
/// Kept to maintain JSON compatibility with previous version: { "emojis": [...] }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EmojiHistoryWrapper {
    #[serde(default)]
    emojis: Vec<EmojiUsage>,
}

/// Manages emoji usage tracking
pub struct EmojiManager {
    /// Recent emojis ordered by recency (index 0 is most recent)
    recent: Vec<EmojiUsage>,
    /// Path to the data directory
    data_dir: PathBuf,
}

impl EmojiManager {
    /// Create a new emoji manager, loading history from disk if available
    pub fn new(data_dir: PathBuf) -> Self {
        let mut manager = Self {
            recent: Vec::with_capacity(MAX_RECENT_EMOJIS),
            data_dir,
        };

        if let Err(e) = manager.load_from_disk() {
            eprintln!("[EmojiManager] Failed to load history: {}", e);
        }

        manager
    }

    /// Record emoji usage (LRU semantics: move to front, increment count)
    pub fn record_usage(&mut self, emoji_char: &str) {
        let now = current_time_millis();

        // Check if emoji exists in recent list
        if let Some(index) = self.recent.iter().position(|e| e.char == emoji_char) {
            // Remove existing entry to update it
            let mut entry = self.recent.remove(index);
            entry.use_count += 1;
            entry.last_used = now;
            self.recent.insert(0, entry);
        } else {
            // Create new entry
            let entry = EmojiUsage {
                char: emoji_char.to_string(),
                use_count: 1,
                last_used: now,
            };
            self.recent.insert(0, entry);
        }

        // Enforce capacity (LRU eviction from end)
        if self.recent.len() > MAX_RECENT_EMOJIS {
            self.recent.truncate(MAX_RECENT_EMOJIS);
        }

        // Persist to disk
        if let Err(e) = self.save_to_disk() {
            eprintln!("[EmojiManager] Failed to save history: {}", e);
        }
    }

    /// Get recent emojis (most recently used first)
    pub fn get_recent(&self) -> Vec<EmojiUsage> {
        self.recent.clone()
    }

    /// Get top N most used emojis
    pub fn get_top_used(&self, n: usize) -> Vec<EmojiUsage> {
        let mut sorted = self.recent.clone();
        // Sort descending by count, then by time
        sorted.sort_by(|a, b| {
            b.use_count
                .cmp(&a.use_count)
                .then_with(|| b.last_used.cmp(&a.last_used))
        });
        sorted.truncate(n);
        sorted
    }

    // --- Persistence Helpers ---

    fn history_path(&self) -> PathBuf {
        self.data_dir.join(EMOJI_HISTORY_FILE)
    }

    fn load_from_disk(&mut self) -> Result<(), String> {
        let path = self.history_path();
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
        let wrapper: EmojiHistoryWrapper =
            serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;

        self.recent = wrapper.emojis;
        
        // Ensure we respect limits even if disk file was modified manually
        if self.recent.len() > MAX_RECENT_EMOJIS {
            self.recent.truncate(MAX_RECENT_EMOJIS);
        }

        eprintln!("[EmojiManager] Loaded {} recent emojis", self.recent.len());
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), String> {
        if !self.data_dir.exists() {
            fs::create_dir_all(&self.data_dir)
                .map_err(|e| format!("Failed to create data dir: {}", e))?;
        }

        let wrapper = EmojiHistoryWrapper {
            emojis: self.recent.clone(),
        };

        let content = serde_json::to_string_pretty(&wrapper)
            .map_err(|e| format!("Serialize error: {}", e))?;

        fs::write(self.history_path(), content).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
}

/// Helper to get current Unix timestamp in milliseconds
fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Default for EmojiManager {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("win11-clipboard-history");
        Self::new(data_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn get_temp_manager(name: &str) -> (EmojiManager, PathBuf) {
        let data_dir = temp_dir().join(name);
        let _ = fs::remove_dir_all(&data_dir); // Ensure clean start
        (EmojiManager::new(data_dir.clone()), data_dir)
    }

    #[test]
    fn test_record_usage_and_ordering() {
        let (mut manager, _dir) = get_temp_manager("emoji_order_test");

        manager.record_usage("A");
        manager.record_usage("B");
        manager.record_usage("A"); // A should move to top with count 2

        let recent = manager.get_recent();
        assert_eq!(recent.len(), 2);
        
        assert_eq!(recent[0].char, "A");
        assert_eq!(recent[0].use_count, 2);
        
        assert_eq!(recent[1].char, "B");
        assert_eq!(recent[1].use_count, 1);
    }

    #[test]
    fn test_lru_eviction() {
        let (mut manager, _dir) = get_temp_manager("emoji_lru_test");

        // Fill past limit
        for i in 0..MAX_RECENT_EMOJIS + 5 {
            manager.record_usage(&format!("emoji_{}", i));
        }

        let recent = manager.get_recent();
        assert_eq!(recent.len(), MAX_RECENT_EMOJIS);
        // The last inserted ("emoji_24") should be at index 0
        assert_eq!(recent[0].char, format!("emoji_{}", MAX_RECENT_EMOJIS + 4));
    }

    #[test]
    fn test_persistence() {
        let (mut manager, dir) = get_temp_manager("emoji_persist_test");

        manager.record_usage("🚀");
        manager.record_usage("🦀");

        // Create new instance pointing to same dir
        let loaded_manager = EmojiManager::new(dir);
        let recent = loaded_manager.get_recent();

        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].char, "🦀");
        assert_eq!(recent[1].char, "🚀");
    }
}