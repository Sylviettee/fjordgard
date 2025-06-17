use std::fmt::Debug;

use reqwest::{
    header::{self, HeaderMap, HeaderValue}, Client, StatusCode
};

pub use error::Error;
use error::Result;
use model::*;
use serde::{Serialize, de::DeserializeOwned};
mod error;
pub mod model;

const USER_AGENT: &str = concat!("fjordgard/", env!("CARGO_PKG_VERSION"));
const UNSPLASH_API_HOST: &str = "https://api.unsplash.com/";

pub struct UnsplashClient {
    client: Client,
}

impl UnsplashClient {
    pub fn new(api_key: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Accept-Version", HeaderValue::from_static("v1"));

        let mut api_key = HeaderValue::from_str(&format!("Client-ID {api_key}"))
            .map_err(|_| Error::InvalidAPIKey)?;
        api_key.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, api_key);

        let client = Client::builder()
            .default_headers(headers)
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self { client })
    }

    async fn request<Q: Serialize, T: DeserializeOwned + Debug>(
        &self,
        route: &str,
        query: Option<Q>,
    ) -> Result<T> {
        let mut req = self.client.get(format!("{UNSPLASH_API_HOST}/{route}"));

        if let Some(ref query) = query {
            req = req.query(query)
        };

        let res = req.send().await?;

        if res.status() == StatusCode::UNAUTHORIZED {
            return Err(Error::InvalidAPIKey)
        }

        let body: UnsplashResponse = res.json().await?;

        match body {
            UnsplashResponse::Error { errors } => Err(Error::Unsplash(errors.join(", "))),
            UnsplashResponse::Success(v) => match serde_json::from_value(v) {
                Ok(o) => Ok(o),
                Err(e) => Err(Error::SerdeJson(e))
            }
        }
    }
}
