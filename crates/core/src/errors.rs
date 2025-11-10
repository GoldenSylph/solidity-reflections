use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ReflectionsError {
    #[error("error during config operation: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("error during IO operation: {0}")]
    IOError(#[from] io::Error),
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("config file is not valid: {0}")]
    Parsing(#[from] toml_edit::TomlError),

    #[error("error writing to config file: {0}")]
    FileWriteError(#[from] io::Error),

    #[error("error parsing config file: {0}")]
    DeserializeError(#[from] toml_edit::de::Error),

    #[error("error generating config file: {0}")]
    SerializeError(#[from] toml_edit::ser::Error),

    #[error("invalid config location: {0}")]
    InvalidLocation(String),
}
