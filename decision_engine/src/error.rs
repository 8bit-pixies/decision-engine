use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("load error: {0:?}")]
    Load(#[from] LoadError),
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("io error: {0:?}")]
    Io(#[from] io::Error),
    #[error("toml deserialization error: {0:?}")]
    Toml(#[from] toml::de::Error),
}
