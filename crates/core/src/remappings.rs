//! Remappings management for Solidity imports

use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Remappings {
    /// Map from import prefix to actual path
    pub mappings: HashMap<String, String>,
}

impl Remappings {
    /// Create a new empty Remappings
    pub fn new() -> Self {
        Self::default()
    }

    /// Load remappings from a remappings.txt file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse remappings from a string
    pub fn parse(content: &str) -> Result<Self> {
        let mut mappings = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((from, to)) = line.split_once('=') {
                mappings.insert(from.trim().to_string(), to.trim().to_string());
            }
        }

        Ok(Self { mappings })
    }

    /// Add a remapping
    pub fn add(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.mappings.insert(from.into(), to.into());
    }

    /// Get the remapped path for a given import
    pub fn remap(&self, import_path: &str) -> String {
        // Try to find the longest matching prefix
        let mut best_match = None;
        let mut best_match_len = 0;

        for (from, to) in &self.mappings {
            if import_path.starts_with(from) && from.len() > best_match_len {
                best_match = Some((from, to));
                best_match_len = from.len();
            }
        }

        if let Some((from, to)) = best_match {
            format!("{}{}", to, &import_path[from.len()..])
        } else {
            import_path.to_string()
        }
    }

    /// Apply remappings to file content (replace import statements)
    pub fn process_imports(&self, content: &str) -> String {
        let mut result = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                // Extract the import path from the import statement
                // Pattern: import { ... } from "path";
                if let Some(start) = line.find('"')
                    && let Some(end) = line[start + 1..].find('"')
                {
                    let import_path = &line[start + 1..start + 1 + end];
                    let remapped = self.remap(import_path);
                    let new_line = line.replace(import_path, &remapped);
                    result.push_str(&new_line);
                    result.push('\n');
                    continue;
                }
            }
            result.push_str(line);
            result.push('\n');
        }

        // Remove trailing newline if original didn't have one
        if !content.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remapping_basic() {
        let mut remappings = Remappings::new();
        remappings.add("@openzeppelin/", "lib/openzeppelin-contracts/");

        assert_eq!(
            remappings.remap("@openzeppelin/contracts/token/ERC20/ERC20.sol"),
            "lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol"
        );
    }

    #[test]
    fn test_remapping_longest_match() {
        let mut remappings = Remappings::new();
        remappings.add("@openzeppelin/", "lib/oz/");
        remappings.add("@openzeppelin/contracts/", "lib/openzeppelin-contracts/contracts/");

        assert_eq!(
            remappings.remap("@openzeppelin/contracts/token/ERC20/ERC20.sol"),
            "lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol"
        );
    }

    #[test]
    fn test_process_imports() {
        let mut remappings = Remappings::new();
        remappings.add("@openzeppelin/", "lib/openzeppelin-contracts/");

        let content = r#"import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";"#;

        let processed = remappings.process_imports(content);

        assert!(processed.contains("lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol"));
        assert!(processed.contains("lib/openzeppelin-contracts/contracts/access/Ownable.sol"));
    }
}
