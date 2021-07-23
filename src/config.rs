use byte_unit::Byte;
use config::{self, ConfigError};
use std::path::PathBuf;

/// Configuration values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Server configuration.
    pub server: ServerConfig,
    /// Paste configuration.
    pub paste: PasteConfig,
}

/// Server configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    /// The socket address to bind.
    pub address: String,
    /// Number of workers to start.
    pub workers: Option<usize>,
    /// Maximum content length.
    pub max_content_length: Byte,
    /// Storage path.
    pub upload_path: PathBuf,
    /// Authentication token.
    pub auth_token: Option<String>,
}

/// Paste configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasteConfig {
    /// Pet names configuration.
    pub pet_names: PetNamesConfig,
    /// Random string configuration.
    pub random: RandomConfig,
    /// Default file extension.
    pub default_extension: String,
}

/// Pet names configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PetNamesConfig {
    /// Use pet names instead of original file names.
    pub enabled: bool,
    /// Count of words that pet name will include.
    pub words: u8,
    /// Separator between the words.
    pub separator: String,
}

/// Random string configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RandomConfig {
    /// Use random strings instead of original file names.
    pub enabled: bool,
    /// Length of the random string to generate.
    pub length: usize,
}

impl Config {
    /// Parses the config file and returns the values.
    pub fn parse(file_name: &str) -> Result<Config, ConfigError> {
        let mut config = config::Config::default();
        config
            .merge(config::File::with_name(file_name))?
            .merge(config::Environment::with_prefix(env!("CARGO_PKG_NAME")).separator("__"))?;
        config.try_into()
    }
}