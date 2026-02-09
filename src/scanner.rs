use std::fs;
use std::path::PathBuf;

use walkdir::WalkDir;

use crate::analyzer::NestingCandidate;

/// Potential risks discovered by scanning a nesting candidate before moving.
#[derive(Debug, Clone)]
pub struct ScanReport {
    /// Files at the destination that would collide with moved items.
    pub collisions: Vec<Collision>,
    /// Symlinks that might break after flattening.
    pub symlink_risks: Vec<SymlinkRisk>,
}

#[derive(Debug, Clone)]
pub struct Collision {
    /// The source path inside the nested directory.
    pub source: PathBuf,
    /// The conflicting path that already exists in the parent.
    pub existing: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SymlinkRisk {
    /// The symlink path.
    pub link: PathBuf,
    /// Where it currently points.
    pub target: PathBuf,
    /// Whether the target lives inside the nested directory being moved.
    pub target_inside_nested: bool,
}

/// Scan a nesting candidate for potential risks before applying a move.
pub fn scan(candidate: &NestingCandidate) -> ScanReport {
    let collisions = detect_collisions(candidate);
    let symlink_risks = detect_symlink_risks(candidate);

    ScanReport {
        collisions,
        symlink_risks,
    }
}

/// Check whether any child in the nested dir would collide with an
/// existing entry in the parent directory.
fn detect_collisions(candidate: &NestingCandidate) -> Vec<Collision> {
    let mut collisions = Vec::new();

    for child in &candidate.children {
        if let Some(name) = child.file_name() {
            let dest = candidate.parent.join(name);
            // The nested directory itself shares the name — skip that.
            if dest == candidate.nested {
                continue;
            }
            if dest.exists() {
                collisions.push(Collision {
                    source: child.clone(),
                    existing: dest,
                });
            }
        }
    }

    collisions
}

/// Walk the nested directory looking for symlinks that reference paths
/// inside the nested tree (which will change after a move).
fn detect_symlink_risks(candidate: &NestingCandidate) -> Vec<SymlinkRisk> {
    let mut risks = Vec::new();

    for entry in WalkDir::new(&candidate.nested)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_symlink()
            && let Ok(target) = fs::read_link(path)
        {
            let target_inside = target.starts_with(&candidate.nested);
            risks.push(SymlinkRisk {
                link: path.to_path_buf(),
                target,
                target_inside_nested: target_inside,
            });
        }
    }

    risks
}

impl ScanReport {
    /// Returns `true` if the scan found no blocking issues.
    pub fn is_safe(&self) -> bool {
        self.collisions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs as unix_fs;
    use tempfile::TempDir;

    fn make_candidate(tmp: &TempDir) -> NestingCandidate {
        let root = tmp.path().join("project");
        let nested = root.join("project");
        fs::create_dir_all(&nested).unwrap();

        fs::write(nested.join("file.txt"), "data").unwrap();
        fs::create_dir(nested.join("src")).unwrap();

        let children = vec![nested.join("file.txt"), nested.join("src")];

        NestingCandidate {
            parent: root.canonicalize().unwrap(),
            nested: root.canonicalize().unwrap().join("project"),
            children,
        }
    }

    #[test]
    fn no_collisions_when_parent_is_clean() {
        let tmp = TempDir::new().unwrap();
        let candidate = make_candidate(&tmp);
        let report = scan(&candidate);
        assert!(report.collisions.is_empty());
        assert!(report.is_safe());
    }

    #[test]
    fn collision_detected() {
        let tmp = TempDir::new().unwrap();
        let candidate = make_candidate(&tmp);

        // Create a conflicting file in the parent
        fs::write(candidate.parent.join("file.txt"), "conflict").unwrap();

        let report = scan(&candidate);
        assert_eq!(report.collisions.len(), 1);
        assert!(!report.is_safe());
    }

    #[test]
    fn symlink_risk_detected() {
        let tmp = TempDir::new().unwrap();
        let candidate = make_candidate(&tmp);

        let link_path = candidate.nested.join("link");
        let target = candidate.nested.join("file.txt");
        unix_fs::symlink(&target, &link_path).unwrap();

        // Re-scan with the symlink present
        let report = scan(&candidate);
        assert!(!report.symlink_risks.is_empty());
    }
}
