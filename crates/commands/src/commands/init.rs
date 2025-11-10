use crate::{
    ConfigLocation,
    utils::{remark, success},
};
use clap::Parser;
use reflections_core::{
    Result,
    config::Paths,
};
use std::fs;

/// Initialize a Foundry project to use Reflections
#[derive(Debug, Clone, Default, Parser, bon::Builder)]
#[allow(clippy::duplicated_attributes)]
#[builder(on(String, into), on(ConfigLocation, into))]
#[clap(after_help = "For more information, read the README.md")]
#[non_exhaustive]
pub struct Init {
    /// Clean the Foundry project by removing .gitmodules and the lib directory
    #[arg(long, default_value_t = false)]
    #[builder(default)]
    pub clean: bool,

    /// Specify the config location.
    ///
    /// This prevents prompting the user if the automatic detection can't determine the config
    /// location.
    #[arg(long, value_enum)]
    pub config_location: Option<ConfigLocation>,
}

pub(crate) async fn init_command(paths: &Paths, cmd: Init) -> Result<()> {
    if cmd.clean {
        remark!("Flag `--clean` was set, cleaning project");
        // Add your cleaning logic here
    }
    
    success!("Reflections initialized successfully!");
    
    let gitignore_path = paths.root.join(".gitignore");
    if gitignore_path.exists() {
        let mut gitignore = fs::read_to_string(&gitignore_path)?;
        if !gitignore.contains("reflections") {
            gitignore.push_str("\n\n# Reflections\n/reflections-output\n");
            fs::write(&gitignore_path, gitignore)?;
        }
    }
    success!("Added `reflections-output` to .gitignore");

    Ok(())
}
