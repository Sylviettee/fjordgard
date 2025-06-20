use std::fmt::Debug;

use bytes::Bytes;
use reqwest::{
    Client, StatusCode,
    header::{self, HeaderMap, HeaderValue},
};

pub use error::Error;
use error::Result;
use model::*;
use serde::{Serialize, de::DeserializeOwned};
mod error;
pub mod model;

const USER_AGENT: &str = concat!("fjordgard/", env!("CARGO_PKG_VERSION"));
const UNSPLASH_API_HOST: &str = "https://api.unsplash.com/";

#[derive(Clone)]
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
    ) -> Result<(T, HeaderMap)> {
        let mut req = self.client.get(format!("{UNSPLASH_API_HOST}/{route}"));

        if let Some(ref query) = query {
            req = req.query(query)
        };

        let res = req.send().await?;

        if res.status() == StatusCode::UNAUTHORIZED {
            return Err(Error::InvalidAPIKey);
        }

        let headers = res.headers().clone();
        let body: UnsplashResponse = res.json().await?;

        match body {
            UnsplashResponse::Error { errors } => Err(Error::Unsplash(errors.join(", "))),
            UnsplashResponse::Success(v) => match serde_json::from_value(v) {
                Ok(o) => Ok((o, headers)),
                Err(e) => Err(Error::SerdeJson(e)),
            },
        }
    }

    // Endpoint: `/collections/:id/photos`
    pub async fn collection_photos(
        &self,
        id: &str,
        opt: Option<CollectionPhotosOptions>,
    ) -> Result<CollectionPhotos> {
        let (photos, headers) = self
            .request(&format!("collections/{id}/photos"), opt)
            .await?;

        Ok(CollectionPhotos {
            collection_total: headers
                .get("X-Total")
                .ok_or(Error::MissingHeader("X-Total"))?
                .to_str()
                .map_err(|_| Error::MalformedResponse)?
                .parse::<usize>()
                .map_err(|_| Error::MalformedResponse)?,
            per_page: headers
                .get("X-Per-Page")
                .ok_or(Error::MissingHeader("X-Per-Page"))?
                .to_str()
                .map_err(|_| Error::MalformedResponse)?
                .parse::<usize>()
                .map_err(|_| Error::MalformedResponse)?,
            photos,
        })
    }

    pub async fn collection(&self, id: &str) -> Result<Collection> {
        let (collection, _) = self
            .request(&format!("collections/{id}"), None::<()>)
            .await?;

        Ok(collection)
    }

    pub async fn download_photo(
        &self,
        photo: &Photo,
        opts: Option<PhotoFetchOptions>,
    ) -> Result<Bytes> {
        let mut req = self.client.get(&photo.urls.raw);

        if let Some(ref query) = opts {
            req = req.query(query);
        }

        Ok(req.send().await?.error_for_status()?.bytes().await?)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    fn api_key() -> String {
        env::var("UNSPLASH_KEY").expect("expected env:UNSPLASH_KEY")
    }

    #[tokio::test]
    async fn collection_photos() {
        let client = UnsplashClient::new(&api_key()).unwrap();
        let collection = client
            .collection_photos(
                "1053828",
                Some(CollectionPhotosOptions {
                    per_page: Some(5),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        assert_eq!(collection.per_page, 5);
    }

    #[tokio::test]
    async fn collection() {
        let client = UnsplashClient::new(&api_key()).unwrap();
        let collection = client.collection("1053828").await.unwrap();

        assert_eq!(collection.title, "Tabliss Official");
    }
}
