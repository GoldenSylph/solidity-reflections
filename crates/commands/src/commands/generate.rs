use crate::utils::{remark, success};
use clap::Parser;
use reflections_core::{
    Result,
    config::Paths,
    generator::{GeneratorOptions, generate_sources_library},
    parser::discover_contracts,
};
use std::fs;

/// Generate Solidity reflection library from contracts
#[derive(Debug, Clone, Parser, bon::Builder)]
#[allow(clippy::duplicated_attributes)]
#[builder(on(String, into))]
#[clap(after_help = "For more information, read the README.md")]
#[non_exhaustive]
pub struct Generate {
    /// Path to the contracts directory (relative to project root)
    #[arg(short, long, default_value = "src")]
    #[builder(default)]
    pub contracts_dir: String,

    /// Output file for generated reflection library
    #[arg(short, long, default_value = "scripts/reflections/di/libraries/Sources.s.sol")]
    #[builder(default)]
    pub output: String,

    /// Name of the generated library
    #[arg(long, default_value = "Sources")]
    #[builder(default)]
    pub library_name: String,

    /// SPDX license identifier
    #[arg(long, default_value = "MIT")]
    #[builder(default)]
    pub license: String,

    /// Solidity version pragma
    #[arg(long, default_value = "^0.8.24")]
    #[builder(default)]
    pub solidity_version: String,
}

pub(crate) async fn generate_command(paths: &Paths, cmd: Generate) -> Result<()> {
    remark!("Discovering contracts in {}", cmd.contracts_dir);

    let contracts = discover_contracts(&paths.root, &cmd.contracts_dir)?;

    if contracts.is_empty() {
        remark!("No contracts found in {}", cmd.contracts_dir);
        return Ok(());
    }

    success!("Found {} contracts", contracts.len());

    for contract in &contracts {
        remark!(
            "  - {} ({})",
            contract.name,
            if contract.is_library {
                "library"
            } else if contract.is_interface {
                "interface"
            } else {
                "contract"
            }
        );
    }

    remark!("Generating reflection library...");

    let options = GeneratorOptions {
        library_name: cmd.library_name.clone(),
        license: cmd.license,
        solidity_version: cmd.solidity_version,
    };

    let output_code = generate_sources_library(&contracts, &options);

    let output_path = paths.root.join(&cmd.output);

    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, output_code)?;

    success!("Generated reflection library at: {}", cmd.output);

    Ok(())
}
