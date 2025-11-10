//! Utility functions used throughout the codebase.
use crate::remappings::Remappings;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Canonicalize a path, stripping Windows UNC prefixes.
pub fn canonicalize_sync(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
    dunce::canonicalize(path.as_ref())
}

/// Recursively copy a directory and all its contents
pub fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Recursively copy a directory and process Solidity files with remappings
pub fn copy_dir_with_remappings(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    remappings: &Remappings,
) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_with_remappings(&src_path, &dst_path, remappings)?;
        } else if src_path.extension().and_then(|s| s.to_str()) == Some("sol") {
            // Process Solidity files with remappings
            let content = fs::read_to_string(&src_path)?;
            let processed = remappings.process_imports(&content);
            fs::write(&dst_path, processed)?;
        } else {
            // Copy other files as-is
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Get the path to the assets directory bundled with the CLI
pub fn get_assets_dir() -> PathBuf {
    // In development, assets are relative to the workspace root
    // In production, they would be bundled with the binary
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));

    // Try workspace structure first (for development)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let workspace_assets = PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("crates").join("commands").join("assets"));
        if let Some(path) = workspace_assets
            && path.exists()
        {
            return path;
        }
    }

    // Fallback to executable directory
    exe_dir.map(|p| p.join("assets")).unwrap_or_else(|| PathBuf::from("assets"))
}
