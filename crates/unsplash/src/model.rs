use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum UnsplashResponse {
    Success(serde_json::Value),
    Error { errors: Vec<String> },
}
