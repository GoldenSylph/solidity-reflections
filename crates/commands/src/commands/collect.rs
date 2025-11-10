use crate::utils::{remark, success};
use clap::Parser;
use reflections_core::{
    Result,
    config::Paths,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

/// Collect ABIs and group by NatSpec tags
#[derive(Debug, Clone, Parser, bon::Builder)]
#[allow(clippy::duplicated_attributes)]
#[builder(on(String, into))]
#[clap(after_help = "For more information, read the README.md")]
#[non_exhaustive]
pub struct Collect {
    /// Path to the forge build output directory
    #[arg(short, long, default_value = "out")]
    #[builder(default)]
    pub artifacts_dir: String,

    /// Output file for collected ABIs
    #[arg(short, long, default_value = "abis.json")]
    #[builder(default)]
    pub output: String,

    /// NatSpec tag to use for grouping in Swagger UI (e.g. @custom:swagger, @title, @notice)
    #[arg(short, long, default_value = "@custom:swagger")]
    #[builder(default)]
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArtifactMetadata {
    abi: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContractMetadata {
    metadata: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectedABI {
    pub contract_name: String,
    pub file_path: String,
    pub abi: serde_json::Value,
    pub group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ABICollection {
    pub grouped: HashMap<String, Vec<CollectedABI>>,
    pub ungrouped: Vec<CollectedABI>,
}

pub(crate) async fn collect_command(paths: &Paths, cmd: Collect) -> Result<()> {
    remark!("Collecting ABIs from {}", cmd.artifacts_dir);

    let artifacts_path = paths.root.join(&cmd.artifacts_dir);
    
    if !artifacts_path.exists() {
        return Err(reflections_core::ReflectionsError::IOError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Artifacts directory not found: {}. Run 'forge build' first.", cmd.artifacts_dir)
        )));
    }

    let mut collection = ABICollection {
        grouped: HashMap::new(),
        ungrouped: Vec::new(),
    };

    // Recursively find all .json files in artifacts directory
    let json_files = find_json_files(&artifacts_path)?;
    
    remark!("Found {} artifact files", json_files.len());

    for json_file in json_files {
        // Skip files that are not contract artifacts (like build-info)
        if json_file.to_string_lossy().contains("build-info") {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&json_file) {
            // Try to parse as artifact with ABI
            if let Ok(artifact) = serde_json::from_str::<ArtifactMetadata>(&content) {
                let contract_name = json_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // Skip if it's just a metadata file without actual contract
                if artifact.abi.as_array().is_none_or(|a| a.is_empty()) {
                    continue;
                }

                // Extract file path relative to artifacts dir
                let file_path = json_file
                    .strip_prefix(&artifacts_path)
                    .unwrap_or(&json_file)
                    .to_string_lossy()
                    .to_string();

                // Try to extract group from metadata
                let group = extract_group_from_metadata(&content, &cmd.tag);

                let collected_abi = CollectedABI {
                    contract_name: contract_name.clone(),
                    file_path,
                    abi: artifact.abi,
                    group: group.clone(),
                };

                if let Some(group_name) = group {
                    collection
                        .grouped
                        .entry(group_name)
                        .or_default()
                        .push(collected_abi);
                } else {
                    collection.ungrouped.push(collected_abi);
                }
            }
        }
    }

    success!(
        "Collected {} contracts ({} grouped, {} ungrouped)",
        collection.grouped.values().map(|v| v.len()).sum::<usize>() + collection.ungrouped.len(),
        collection.grouped.values().map(|v| v.len()).sum::<usize>(),
        collection.ungrouped.len()
    );

    remark!("Groups found:");
    for (group, contracts) in &collection.grouped {
        remark!("  - {} ({} contracts)", group, contracts.len());
    }

    // Write output
    let output_path = paths.root.join(&cmd.output);
    let output_json = serde_json::to_string_pretty(&collection).map_err(|e| {
        reflections_core::ReflectionsError::IOError(std::io::Error::other(
            format!("Failed to serialize ABIs to JSON: {e}")
        ))
    })?;
    fs::write(&output_path, output_json)?;

    success!("ABIs collected and saved to: {}", cmd.output);

    Ok(())
}

fn find_json_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut json_files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                json_files.extend(find_json_files(&path)?);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                json_files.push(path);
            }
        }
    }

    Ok(json_files)
}

fn extract_group_from_metadata(content: &str, tag: &str) -> Option<String> {
    // Try to parse metadata field
    if let Ok(contract_meta) = serde_json::from_str::<ContractMetadata>(content)
        && let Ok(metadata_obj) = serde_json::from_str::<serde_json::Value>(&contract_meta.metadata)
    {
        // Navigate to output.devdoc and output.userdoc
        if let Some(output) = metadata_obj.get("output") {
            // Check devdoc first
            if let Some(devdoc) = output.get("devdoc")
                && let Some(group) = extract_tag_from_doc(devdoc, tag)
            {
                return Some(group);
            }

            // Check userdoc
            if let Some(userdoc) = output.get("userdoc")
                && let Some(group) = extract_tag_from_doc(userdoc, tag)
            {
                return Some(group);
            }
        }
    }

    None
}

fn extract_tag_from_doc(doc: &serde_json::Value, tag: &str) -> Option<String> {
    // Handle different tag formats
    match tag {
        "@title" => {
            doc.get("title").and_then(|v| v.as_str()).map(String::from)
        }
        "@notice" => {
            doc.get("notice").and_then(|v| v.as_str()).map(String::from)
        }
        custom_tag if custom_tag.starts_with("@custom:") => {
            let custom_key = custom_tag.strip_prefix("@custom:").expect("starts_with checked");
            doc.get("custom")
                .and_then(|c| c.get(custom_key))
                .and_then(|v| v.as_str())
                .map(String::from)
        }
        _ => None,
    }
}
