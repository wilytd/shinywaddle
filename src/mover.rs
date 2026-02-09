use std::fs;
use std::path::PathBuf;

use log::info;

use crate::analyzer::NestingCandidate;
use crate::scanner::{self, ScanReport};
use crate::{Error, Result};

/// Result of applying a flatten operation.
#[derive(Debug)]
pub struct MoveResult {
    /// Items successfully moved from nested -> parent.
    pub moved: Vec<MoveRecord>,
}

/// A single item that was moved.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MoveRecord {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Plan and optionally execute a flatten operation.
///
/// When `dry_run` is true, no filesystem changes are made — the function
/// returns what *would* happen.
pub fn flatten(candidate: &NestingCandidate, dry_run: bool) -> Result<MoveResult> {
    let report: ScanReport = scanner::scan(candidate);

    if !report.collisions.is_empty() {
        let first = &report.collisions[0];
        return Err(Error::Collision {
            existing: first.existing.clone(),
        });
    }

    if !report.symlink_risks.is_empty() {
        for risk in &report.symlink_risks {
            log::warn!(
                "symlink risk: {} -> {} (target inside nested: {})",
                risk.link.display(),
                risk.target.display(),
                risk.target_inside_nested,
            );
        }
    }

    let mut moved = Vec::new();

    for child in &candidate.children {
        let name = child
            .file_name()
            .ok_or_else(|| Error::Other(format!("no filename for {}", child.display())))?;
        let dest = candidate.parent.join(name);

        if dest == candidate.nested {
            // Skip the nested directory entry itself; we'll remove it after.
            continue;
        }

        if !dry_run {
            fs::rename(child, &dest).map_err(|e| Error::Io {
                path: child.clone(),
                source: e,
            })?;
            info!("moved {} -> {}", child.display(), dest.display());
        }

        moved.push(MoveRecord {
            from: child.clone(),
            to: dest,
        });
    }

    // Remove the now-empty nested directory.
    if !dry_run {
        fs::remove_dir(&candidate.nested).map_err(|e| Error::Io {
            path: candidate.nested.clone(),
            source: e,
        })?;
        info!("removed empty directory {}", candidate.nested.display());
    }

    Ok(MoveResult { moved })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, NestingCandidate) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("project");
        let nested = root.join("project");

        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("file.txt"), "data").unwrap();
        fs::create_dir(nested.join("src")).unwrap();

        let root_canon = root.canonicalize().unwrap();
        let nested_canon = root_canon.join("project");

        let children = vec![nested_canon.join("file.txt"), nested_canon.join("src")];

        let candidate = NestingCandidate {
            parent: root_canon,
            nested: nested_canon,
            children,
        };

        (tmp, candidate)
    }

    #[test]
    fn dry_run_does_not_modify_filesystem() {
        let (_tmp, candidate) = setup();
        let result = flatten(&candidate, true).unwrap();

        assert_eq!(result.moved.len(), 2);
        // Nested dir should still exist
        assert!(candidate.nested.exists());
    }

    #[test]
    fn apply_moves_files() {
        let (_tmp, candidate) = setup();
        let result = flatten(&candidate, false).unwrap();

        assert_eq!(result.moved.len(), 2);
        // Nested dir should be removed
        assert!(!candidate.nested.exists());
        // Files should be in parent
        assert!(candidate.parent.join("file.txt").exists());
        assert!(candidate.parent.join("src").is_dir());
    }

    #[test]
    fn collision_aborts() {
        let (_tmp, candidate) = setup();

        // Create conflicting file in parent
        fs::write(candidate.parent.join("file.txt"), "conflict").unwrap();

        let err = flatten(&candidate, false).unwrap_err();
        assert!(matches!(err, Error::Collision { .. }));
        // Nested dir should still exist — nothing was moved
        assert!(candidate.nested.exists());
    }
}
