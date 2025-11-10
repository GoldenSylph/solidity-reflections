//! Manage the Reflections configuration
use crate::errors::ConfigError;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, ConfigError>;

/// The paths used by Reflections.
///
/// The paths are canonicalized on creation of the object.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Paths {
    /// The root directory of the project.
    pub root: PathBuf,

    /// The path to the config file (foundry.toml or reflections.toml).
    pub config: PathBuf,

    /// The path to the remappings file (does not need to exist).
    pub remappings: PathBuf,
}

impl Paths {
    /// Instantiate all the paths needed for Reflections.
    pub fn new() -> Result<Self> {
        Self::with_config(None)
    }

    /// Instantiate all the paths with a specific config location.
    pub fn with_config(config_location: Option<ConfigLocation>) -> Result<Self> {
        let root = dunce::canonicalize(Self::get_root_path())?;
        Self::with_root_and_config(root, config_location)
    }

    /// Instantiate all the paths with a specific root and config location.
    pub fn with_root_and_config(
        root: impl AsRef<Path>,
        config_location: Option<ConfigLocation>,
    ) -> Result<Self> {
        let root = root.as_ref();
        let config = Self::get_config_path(root, config_location)?;
        let remappings = root.join("remappings.txt");

        Ok(Self { root: root.to_path_buf(), config, remappings })
    }

    /// Get the root directory path.
    pub fn get_root_path() -> PathBuf {
        env::var("REFLECTIONS_PROJECT_ROOT").map_or_else(
            |_| {
                debug!("REFLECTIONS_PROJECT_ROOT not defined, using current directory");
                env::current_dir().expect("could not get current directory")
            },
            |p| {
                if p.is_empty() {
                    debug!("REFLECTIONS_PROJECT_ROOT exists but is empty, using current directory");
                    env::current_dir().expect("could not get current directory")
                } else {
                    debug!(path = p; "root set by REFLECTIONS_PROJECT_ROOT");
                    PathBuf::from(p)
                }
            },
        )
    }

    /// Get the path to the config file.
    fn get_config_path(
        root: impl AsRef<Path>,
        config_location: Option<ConfigLocation>,
    ) -> Result<PathBuf> {
        let location = config_location
            .or_else(|| detect_config_location(root.as_ref()))
            .unwrap_or(ConfigLocation::Foundry);

        Ok(match location {
            ConfigLocation::Foundry => root.as_ref().join("foundry.toml"),
            ConfigLocation::Reflections => root.as_ref().join("reflections.toml"),
        })
    }
}

/// The location where the config should be stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigLocation {
    /// Use foundry.toml
    Foundry,
    /// Use reflections.toml
    Reflections,
}

impl std::str::FromStr for ConfigLocation {
    type Err = ConfigError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "foundry" => Ok(Self::Foundry),
            "reflections" => Ok(Self::Reflections),
            _ => Err(ConfigError::InvalidLocation(s.to_string())),
        }
    }
}

/// Auto-detect the config location based on file existence.
pub fn detect_config_location(root: impl AsRef<Path>) -> Option<ConfigLocation> {
    let foundry_path = root.as_ref().join("foundry.toml");
    let reflections_path = root.as_ref().join("reflections.toml");

    if foundry_path.exists() {
        Some(ConfigLocation::Foundry)
    } else if reflections_path.exists() {
        Some(ConfigLocation::Reflections)
    } else {
        None
    }
}

/// Reflections configuration structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ReflectionsConfig {
    /// OpenZeppelin contracts version
    #[serde(default = "default_openzeppelin_version")]
    pub openzeppelin_version: String,
    
    /// zkSync-OS repository URL
    #[serde(default = "default_zksync_os_url")]
    pub zksync_os_url: String,
}

fn default_openzeppelin_version() -> String {
    "v5.1.0".to_string()
}

fn default_zksync_os_url() -> String {
    "https://github.com/matter-labs/zksync-os".to_string()
}

impl Default for ReflectionsConfig {
    fn default() -> Self {
        Self {
            openzeppelin_version: default_openzeppelin_version(),
            zksync_os_url: default_zksync_os_url(),
        }
    }
}

impl ReflectionsConfig {
    /// Load configuration from reflections.toml file
    pub fn load(config_path: impl AsRef<Path>) -> Result<Self> {
        let config_path = config_path.as_ref();
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(config_path)?;
        
        toml_edit::de::from_str(&content).map_err(ConfigError::DeserializeError)
    }

    /// Save configuration to reflections.toml file
    pub fn save(&self, config_path: impl AsRef<Path>) -> Result<()> {
        let content = toml_edit::ser::to_string_pretty(self)
            .map_err(ConfigError::SerializeError)?;
        
        std::fs::write(config_path.as_ref(), content)
            .map_err(ConfigError::FileWriteError)?;
        
        Ok(())
    }
}
