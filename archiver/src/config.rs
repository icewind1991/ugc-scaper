use secretfile::SecretError;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Error reading config from {path}: {error:#}")]
    Read { path: String, error: std::io::Error },
    #[error("Error parsing config from {path}: {error:#}")]
    Parse {
        path: String,
        error: toml::de::Error,
    },
    #[error("Error reading password from file: {0:#}")]
    PasswordSecret(SecretError),
}

#[derive(Deserialize)]
pub struct Config {
    pub db: DBConfig,
    pub api: ApiConfig,
}

impl Config {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let raw = read_to_string(path).map_err(|error| ConfigError::Read {
            path: path.display().to_string(),
            error,
        })?;
        toml::from_str(&raw).map_err(|error| ConfigError::Parse {
            path: path.display().to_string(),
            error,
        })
    }
}

#[derive(Deserialize)]
pub struct ApiConfig {
    pub url: String,
}

#[derive(Deserialize)]
pub struct DBConfig {
    pub url: String,
    password_file: String,
}

impl DBConfig {
    pub fn password(&self) -> Result<String, ConfigError> {
        secretfile::load(&self.password_file).map_err(ConfigError::PasswordSecret)
    }
}
