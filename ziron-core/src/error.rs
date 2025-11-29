//! Error types for ziron-core

use thiserror::Error;

/// Result type alias for ziron-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for ziron-core
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Theme error: {0}")]
    Theme(String),

    #[error("IPC error: {0}")]
    Ipc(String),
}

