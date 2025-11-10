//! High-level commands for the Reflections CLI
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use crate::commands::{Args, Command};
use clap::builder::PossibleValue;
pub use clap_verbosity_flag::Verbosity;
use clap_verbosity_flag::log::Level;
use commands::CustomLevel;
use derive_more::derive::FromStr;
use reflections_core::{Result, config::Paths};
use std::{
    env,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use utils::{get_config_location, intro, outro, outro_cancel, step};

pub mod commands;
pub mod utils;

static TUI_ENABLED: AtomicBool = AtomicBool::new(true);

/// The location where the Reflections config should be stored.
///
/// This is a new type so we can implement the `ValueEnum` trait for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromStr)]
pub struct ConfigLocation(reflections_core::config::ConfigLocation);

impl clap::ValueEnum for ConfigLocation {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self(reflections_core::config::ConfigLocation::Foundry),
            Self(reflections_core::config::ConfigLocation::Reflections),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self.0 {
            reflections_core::config::ConfigLocation::Foundry => PossibleValue::new("foundry"),
            reflections_core::config::ConfigLocation::Reflections => PossibleValue::new("reflections"),
        })
    }
}

impl From<ConfigLocation> for reflections_core::config::ConfigLocation {
    fn from(value: ConfigLocation) -> Self {
        value.0
    }
}

impl From<reflections_core::config::ConfigLocation> for ConfigLocation {
    fn from(value: reflections_core::config::ConfigLocation) -> Self {
        Self(value)
    }
}

pub async fn run(command: Command, verbosity: Verbosity<CustomLevel>) -> Result<()> {
    if env::var("RUST_LOG").is_ok() {
        env_logger::builder().try_init().ok(); // init logger if possible (not already initialized)
        TUI_ENABLED.store(false, Ordering::Relaxed);
    } else {
        match verbosity.log_level() {
            Some(level) if level > Level::Error => {
                // the user requested structure logging (-v[v*])
                // init logger if possible (not already initialized)
                env_logger::Builder::new()
                    .filter_level(verbosity.log_level_filter())
                    .try_init()
                    .ok();
                TUI_ENABLED.store(false, Ordering::Relaxed);
            }
            Some(_) => TUI_ENABLED.store(true, Ordering::Relaxed),
            _ => TUI_ENABLED.store(false, Ordering::Relaxed),
        }
    }
    match command {
        Command::Init(cmd) => {
            intro!("✨ Reflections Init ✨");
            step!("Initialize Foundry project to use Reflections");
            // for init, we always use the current dir as root, unless specified by env
            let root = env::var("REFLECTIONS_PROJECT_ROOT")
                .ok()
                .filter(|p| !p.is_empty())
                .map_or(env::current_dir()?, PathBuf::from);
            
            let config_loc = if let Some(loc) = cmd.config_location {
                loc.into()
            } else {
                get_config_location()?.into()
            };
            
            let paths = Paths::with_root_and_config(&root, Some(config_loc))?;
            commands::init::init_command(&paths, cmd).await.inspect_err(|_| {
                outro_cancel!("An error occurred during initialization");
            })?;
            outro!("Done initializing!");
        }
        Command::Version(_) => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("reflections {VERSION}");
        }
    }
    Ok(())
}
