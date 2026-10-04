use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("degenerate geometry: {0}")]
    DegenerateGeometry(String),

    #[error("invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("topology violation: {0}")]
    TopologyViolation(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type GeoResult<T> = Result<T, GeoError>;
