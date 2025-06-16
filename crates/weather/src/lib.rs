use std::fmt::Debug;

use reqwest::Client;

pub use error::*;
use model::*;
use serde::{Serialize, de::DeserializeOwned};

mod error;
mod model;

static USER_AGENT: &str = concat!("fjordgard/", env!("CARGO_PKG_VERSION"));
const GEOCODING_API_HOST: &str = "geocoding-api.open-meteo.com";

pub struct MeteoClient {
    api_key: Option<String>,
    client: Client,
}

impl MeteoClient {
    pub fn new(api_key: Option<&str>) -> Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self {
            api_key: api_key.map(|k| k.to_string()),
            client,
        })
    }

    async fn request<O1: Serialize, O2: Serialize, T: DeserializeOwned + Debug>(
        &self,
        url: &str,
        route: &str,
        opt1: Option<O1>,
        opt2: Option<O2>,
    ) -> Result<T> {
        let prefix = if self.api_key.is_some() {
            "customer-"
        } else {
            ""
        };

        let mut req = self.client.get(format!("https://{prefix}{url}/v1/{route}"));

        if let Some(ref key) = self.api_key {
            req = req.query(&[("apikey", key)])
        };

        if let Some(ref opt) = opt1 {
            req = req.query(opt)
        };

        if let Some(ref opt) = opt2 {
            req = req.query(opt)
        };

        let resp: MeteoResponse<T> = req.send().await?.json().await?;

        if resp.error.unwrap_or_default() {
            return Err(Error::Meteo(resp.reason.unwrap_or_default()));
        }

        if let Some(res) = resp.results {
            Ok(res)
        } else {
            Err(Error::MeteoEmpty)
        }
    }

    /// Endpoint: `/search`
    pub async fn geocode(
        &self,
        name: &str,
        opt: Option<GeocodeOptions>,
    ) -> Result<Vec<GeocodeResponse>> {
        let resp = self
            .request(GEOCODING_API_HOST, "search", Some(&[("name", name)]), opt)
            .await;

        match resp {
            Err(Error::MeteoEmpty) => Ok(vec![]),
            Err(e) => Err(e),
            Ok(o) => Ok(o),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn geocode() {
        let client = MeteoClient::new(None).unwrap();
        let res = client
            .geocode("London, United Kingdom", None)
            .await
            .unwrap();
        let london = res.get(0).unwrap();

        assert_eq!(london.timezone, "Europe/London");
        assert_eq!(london.admin1, Some("England".to_string()));
        assert_eq!(london.country, "United Kingdom");
    }
}
