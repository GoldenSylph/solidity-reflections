use crate::utils::{remark, success};
use clap::Parser;
use reflections_core::{
    Result,
    config::{Paths, ReflectionsConfig},
    remappings::Remappings,
    utils::{copy_dir_with_remappings, get_assets_dir},
};
use std::fs;

/// Initialize a Foundry project to use Reflections
#[derive(Debug, Clone, Default, Parser, bon::Builder)]
#[allow(clippy::duplicated_attributes)]
#[builder(on(String, into))]
#[clap(after_help = "For more information, read the README.md")]
#[non_exhaustive]
pub struct Init {
    /// Clean the Foundry project by removing previous Reflections scaffolding before re-initializing
    #[arg(long, default_value_t = false)]
    #[builder(default)]
    pub clean: bool,

    /// OpenZeppelin contracts version to use
    #[arg(long, default_value = "v5.1.0")]
    #[builder(default)]
    pub openzeppelin_version: String,

    /// zkSync-OS repository URL
    #[arg(long, default_value = "https://github.com/matter-labs/zksync-os")]
    #[builder(default)]
    pub zksync_os_url: String,
}

pub(crate) async fn init_command(paths: &Paths, cmd: Init) -> Result<()> {
    // Load or create configuration
    let reflections_config_path = paths.root.join("reflections.toml");
    let mut config = ReflectionsConfig::load(&reflections_config_path)?;
    
    // Update config with CLI arguments (CLI args override config file)
    config.openzeppelin_version = cmd.openzeppelin_version;
    config.zksync_os_url = cmd.zksync_os_url;
    
    // Save updated configuration
    config.save(&reflections_config_path)?;
    remark!("Updated reflections.toml with configuration");

    if cmd.clean {
        remark!("Flag `--clean` was set, cleaning project");
        
        // Remove previous DI framework scaffolding
        let reflections_dir = paths.root.join("scripts").join("reflections");
        if reflections_dir.exists() {
            fs::remove_dir_all(&reflections_dir)?;
            remark!("Removed previous scripts/reflections/ directory");
        }
    }

    // Copy DI framework from assets to project
    let assets_dir = get_assets_dir();
    let solidity_assets = assets_dir.join("solidity");

    if !solidity_assets.exists() {
        return Err(reflections_core::ReflectionsError::IOError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Assets directory not found at: {}", solidity_assets.display()),
        )));
    }

    let target_dir = paths.root.join("scripts").join("reflections").join("di");
    remark!("Copying DI framework to scripts/reflections/di/...");

    // Load or create default remappings
    let remappings = if paths.remappings.exists() {
        remark!("Loading remappings from {}", paths.remappings.display());
        Remappings::from_file(&paths.remappings)?
    } else {
        remark!("No remappings.txt found, using default mappings");
        let mut remappings = Remappings::new();
        // Default remapping: strip the hardcoded prefixes from assets
        // The "src/scripts/reflections/" prefix in assets will be replaced to keep them local
        remappings.add("src/scripts/reflections/", "");
        // zksync-os prefix for Matter Labs dependencies
        remappings.add("zksync-os/", "");
        remappings
    };

    copy_dir_with_remappings(&solidity_assets, &target_dir, &remappings)?;
    success!("DI framework scaffolded successfully!");

    // Update .gitignore using template
    let gitignore_template_path = assets_dir.join(".gitignoreTemplate");
    let gitignore_template = if gitignore_template_path.exists() {
        fs::read_to_string(&gitignore_template_path)?
    } else {
        // Fallback if template is missing
        "# Reflections - DI Framework\n# The entire scripts/reflections/di/ directory is scaffolded by `reflections init`\n# and should not be committed to version control\n/scripts/reflections/\n".to_string()
    };

    let gitignore_path = paths.root.join(".gitignore");
    if gitignore_path.exists() {
        let mut gitignore = fs::read_to_string(&gitignore_path)?;
        if !gitignore.contains("# Reflections") {
            gitignore.push_str("\n\n");
            gitignore.push_str(&gitignore_template);
            fs::write(&gitignore_path, gitignore)?;
            success!("Added generated Sources.s.sol to .gitignore");
        }
    } else {
        // Create .gitignore if it doesn't exist
        fs::write(&gitignore_path, gitignore_template)?;
        success!("Created .gitignore with Reflections entries");
    }

    success!("Reflections initialized successfully!");
    remark!("Next step: Run `reflections generate` to create your Sources library");

    Ok(())
}
