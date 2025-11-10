//! Utility functions used throughout the codebase.
use std::path::{Path, PathBuf};

/// Canonicalize a path, stripping Windows UNC prefixes.
pub fn canonicalize_sync(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
    dunce::canonicalize(path.as_ref())
}

