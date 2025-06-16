#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("meteo error: {0}")]
    Meteo(String),
    #[error("json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to parse pressure level")]
    InvalidPressureLevel,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
