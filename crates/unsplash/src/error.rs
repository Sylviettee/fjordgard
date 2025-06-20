#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("invalid API key provided")]
    InvalidAPIKey,
    #[error("unsplash error: {0}")]
    Unsplash(String),
    #[error("json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("unsplash response missing header: {0}")]
    MissingHeader(&'static str),
    #[error("unsplash response malformed")]
    MalformedResponse,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
