use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Route discovery failed: {0}")]
    RouteDiscoveryFailed(String),

    #[error("Build failed for route {route}: {message}")]
    RouteBuildFailed { route: String, message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),
}

pub type Result<T> = std::result::Result<T, BuildError>;
