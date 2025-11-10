//! Solidity contract parser to extract contract names from source files

use crate::errors::ReflectionsError;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub name: String,
    pub path: PathBuf,
    pub import_path: String,
    pub is_library: bool,
    pub is_interface: bool,
}

pub fn discover_contracts(
    root: &Path,
    src_dir: &str,
) -> Result<Vec<ContractInfo>, ReflectionsError> {
    let src_path = root.join(src_dir);
    if !src_path.exists() {
        return Err(ReflectionsError::IOError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory not found: {}", src_path.display()),
        )));
    }

    let contract_regex = Regex::new(
        r"(?m)^\s*(contract|library|interface)\s+([A-Za-z_][A-Za-z0-9_]*)",
    )
    .map_err(|e| {
        ReflectionsError::IOError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Regex error: {e}"),
        ))
    })?;

    let mut contracts = Vec::new();

    for entry in WalkDir::new(&src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sol"))
    {
        let path = entry.path();
        let content = std::fs::read_to_string(path)?;

        for cap in contract_regex.captures_iter(&content) {
            let contract_type = &cap[1];
            let name = cap[2].to_string();

            // Calculate import path relative to src directory
            let relative_path = path.strip_prefix(&src_path).map_err(|e| {
                ReflectionsError::IOError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Path error: {e}"),
                ))
            })?;

            let import_path = format!("{}/{}", src_dir, relative_path.display()).replace('\\', "/");

            contracts.push(ContractInfo {
                name,
                path: path.to_path_buf(),
                import_path,
                is_library: contract_type == "library",
                is_interface: contract_type == "interface",
            });
        }
    }

    contracts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(contracts)
}
