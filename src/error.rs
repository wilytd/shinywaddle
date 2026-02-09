use std::path::PathBuf;

/// Alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors produced by fs-cleaner.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("collision: {existing} already exists at destination")]
    Collision { existing: PathBuf },

    #[error("permission denied: {path}")]
    Permission { path: PathBuf },

    #[error("symlink would break: {link} -> {target}")]
    BrokenSymlink { link: PathBuf, target: PathBuf },

    #[error("cross-device move not yet supported: {path}")]
    CrossDevice { path: PathBuf },

    #[error("{0}")]
    Other(String),
}
