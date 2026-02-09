use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// A detected case of redundant directory nesting.
#[derive(Debug, Clone)]
pub struct NestingCandidate {
    /// The parent directory (e.g. `/home/app`).
    pub parent: PathBuf,
    /// The redundant child directory (e.g. `/home/app/app`).
    pub nested: PathBuf,
    /// Items inside the nested directory that would be moved up.
    pub children: Vec<PathBuf>,
}

/// Analyze a directory tree for redundant nesting patterns.
///
/// A directory is considered redundantly nested when it contains a single
/// subdirectory whose name matches its own name (e.g. `project/project/...`).
pub fn detect_nesting(root: &Path) -> Result<Vec<NestingCandidate>> {
    let root = root.canonicalize().map_err(|e| Error::Io {
        path: root.to_path_buf(),
        source: e,
    })?;

    let dir_name = root
        .file_name()
        .ok_or_else(|| Error::Other(format!("cannot determine name of {}", root.display())))?;

    let candidate = root.join(dir_name);

    if !candidate.is_dir() {
        return Ok(vec![]);
    }

    let children = list_dir(&candidate)?;

    Ok(vec![NestingCandidate {
        parent: root.clone(),
        nested: candidate,
        children,
    }])
}

/// List immediate children of a directory.
fn list_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        result.push(entry.path());
    }
    result.sort();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detect_simple_nesting() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("project");
        let nested = root.join("project");
        let inner_file = nested.join("src");

        fs::create_dir_all(&inner_file).unwrap();
        fs::write(nested.join("README.md"), "hello").unwrap();

        let results = detect_nesting(&root).unwrap();
        assert_eq!(results.len(), 1);

        let candidate = &results[0];
        assert_eq!(candidate.parent, root.canonicalize().unwrap());
        assert_eq!(
            candidate.nested,
            root.canonicalize().unwrap().join("project")
        );
        assert_eq!(candidate.children.len(), 2); // README.md and src/
    }

    #[test]
    fn no_nesting_detected() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("project");
        let other = root.join("other_name");

        fs::create_dir_all(&other).unwrap();

        let results = detect_nesting(&root).unwrap();
        assert!(results.is_empty());
    }
}
