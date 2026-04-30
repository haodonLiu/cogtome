//! Assembly call statistics and zombie detection.
//!
//! Tracks call counts and last-used timestamps for each assembly.
//! Assemblies with 0 calls in 30 days are marked as zombies and can be archived.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Per-assembly call statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyStats {
    pub call_count: u64,
    pub last_called_at: Option<u64>,
    pub created_at: u64,
    pub zombie_since: Option<u64>,
}

impl Default for AssemblyStats {
    fn default() -> Self {
        Self {
            call_count: 0,
            last_called_at: None,
            created_at: now_secs(),
            zombie_since: None,
        }
    }
}

/// Global stats store
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsStore {
    pub assemblies: HashMap<String, AssemblyStats>,
}

const ZOMBIE_DAYS: u64 = 30;
const ARCHIVE_DAYS: u64 = 90;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl StatsStore {
    /// Load stats from disk, or create empty store
    pub fn load() -> Self {
        let path = Self::stats_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(store) => store,
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "stats file corrupted, starting fresh");
                        Self::default()
                    }
                },
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "failed to read stats file, starting fresh");
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Save stats to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::stats_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn stats_path() -> PathBuf {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(".cogtome")
            .join("stats.json")
    }

    /// Record a call to an assembly
    pub fn record_call(&mut self, assembly: &str) {
        let stats = self.assemblies.entry(assembly.to_string()).or_default();
        stats.call_count += 1;
        stats.last_called_at = Some(now_secs());
        if stats.zombie_since.is_some() {
            debug!(assembly = %assembly, "zombie revived by call");
            stats.zombie_since = None;
        }
    }

    /// Mark newly zombie assemblies (30+ days with 0 calls)
    pub fn update_zombie_status(&mut self) {
        let now = now_secs();
        for (_, stats) in self.assemblies.iter_mut() {
            match stats.last_called_at {
                None => {
                    // Never called — check if older than zombie threshold
                    if now - stats.created_at > ZOMBIE_DAYS * 86400 && stats.zombie_since.is_none() {
                        stats.zombie_since = Some(now);
                    }
                }
                Some(last) => {
                    if now - last > ZOMBIE_DAYS * 86400 && stats.zombie_since.is_none() {
                        stats.zombie_since = Some(now);
                    }
                }
            }
        }
    }

    /// Get assemblies eligible for archive (zombie for 90+ days)
    pub fn auto_archivable(&self) -> Vec<&str> {
        let now = now_secs();
        self.assemblies
            .iter()
            .filter(|(_, stats)| {
                stats.zombie_since.map_or(false, |z| now - z > ARCHIVE_DAYS * 86400)
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }

}

/// Stats row for display
#[derive(Debug)]
pub struct StatsRow {
    pub name: String,
    pub call_count: u64,
    pub last_used: String,
    pub status: String,
}

impl StatsStore {
    /// Generate display rows for cogtome stats
    pub fn display_rows(&self) -> Vec<StatsRow> {
        let now = now_secs();
        let mut rows: Vec<StatsRow> = self.assemblies
            .iter()
            .map(|(name, stats)| {
                let last_used = match stats.last_called_at {
                    None => "never".to_string(),
                    Some(t) => {
                        let elapsed = now.saturating_sub(t);
                        format_duration(elapsed)
                    }
                };

                let status = if stats.zombie_since.is_some() {
                    "zombie".to_string()
                } else if stats.call_count == 0 {
                    "idle".to_string()
                } else {
                    "active".to_string()
                };

                StatsRow {
                    name: name.clone(),
                    call_count: stats.call_count,
                    last_used,
                    status,
                }
            })
            .collect();

        rows.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        rows
    }
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_call() {
        let mut store = StatsStore::default();
        store.record_call("test-asm");
        assert_eq!(store.assemblies["test-asm"].call_count, 1);
        store.record_call("test-asm");
        assert_eq!(store.assemblies["test-asm"].call_count, 2);
    }

    #[test]
    fn test_zombie_detection() {
        let mut store = StatsStore::default();
        let old_time = now_secs() - 31 * 86400;
        store.assemblies.insert("old-asm".to_string(), AssemblyStats {
            call_count: 5,
            last_called_at: Some(old_time),
            created_at: old_time - 86400,
            zombie_since: None,
        });
        store.update_zombie_status();
        assert!(store.assemblies["old-asm"].zombie_since.is_some());
    }

    #[test]
    fn test_revive_zombie() {
        let mut store = StatsStore::default();
        store.assemblies.insert("dead-asm".to_string(), AssemblyStats {
            call_count: 0,
            last_called_at: None,
            created_at: now_secs() - 31 * 86400,
            zombie_since: Some(now_secs()),
        });
        store.record_call("dead-asm");
        assert!(store.assemblies["dead-asm"].zombie_since.is_none());
        assert_eq!(store.assemblies["dead-asm"].call_count, 1);
    }
}
