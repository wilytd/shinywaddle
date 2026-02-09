use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mover::MoveRecord;
use crate::{Error, Result};

const JOURNAL_FILE: &str = ".fs-cleaner-journal.json";

/// Persistent record of moves performed, enabling rollback.
#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub entries: Vec<MoveRecord>,
}

impl Journal {
    /// Create a new empty journal.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Record a batch of moves.
    pub fn record(&mut self, moves: Vec<MoveRecord>) {
        self.entries.extend(moves);
    }

    /// Write the journal to disk alongside the target directory.
    pub fn save(&self, dir: &Path) -> Result<PathBuf> {
        let path = dir.join(JOURNAL_FILE);
        let json =
            serde_json::to_string_pretty(&self.entries).map_err(|e| Error::Other(e.to_string()))?;
        fs::write(&path, json).map_err(|e| Error::Io {
            path: path.clone(),
            source: e,
        })?;
        Ok(path)
    }

    /// Load a journal from disk.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(JOURNAL_FILE);
        let data = fs::read_to_string(&path).map_err(|e| Error::Io {
            path: path.clone(),
            source: e,
        })?;
        let entries: Vec<MoveRecord> =
            serde_json::from_str(&data).map_err(|e| Error::Other(e.to_string()))?;
        Ok(Self { entries })
    }

    /// Reverse all recorded moves (last-in, first-out).
    pub fn rollback(&self) -> Result<usize> {
        let mut count = 0;
        for record in self.entries.iter().rev() {
            if record.to.exists() {
                fs::rename(&record.to, &record.from).map_err(|e| Error::Io {
                    path: record.to.clone(),
                    source: e,
                })?;
                count += 1;
            }
        }
        Ok(count)
    }
}

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mut journal = Journal::new();
        journal.record(vec![MoveRecord {
            from: PathBuf::from("/a/b"),
            to: PathBuf::from("/a/c"),
        }]);

        let saved = journal.save(tmp.path()).unwrap();
        assert!(saved.exists());

        let loaded = Journal::load(tmp.path()).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].from, PathBuf::from("/a/b"));
    }

    #[test]
    fn rollback_reverses_moves() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("original");
        let dest = tmp.path().join("moved");

        fs::write(&dest, "data").unwrap();

        let journal = Journal {
            entries: vec![MoveRecord {
                from: src.clone(),
                to: dest.clone(),
            }],
        };

        let count = journal.rollback().unwrap();
        assert_eq!(count, 1);
        assert!(src.exists());
        assert!(!dest.exists());
    }
}
