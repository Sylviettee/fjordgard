use std::fmt::Debug;

use reqwest::Client;

pub use error::*;
use model::*;
use serde::{Serialize, de::DeserializeOwned};

mod error;
mod model;

const USER_AGENT: &str = concat!("fjordgard/", env!("CARGO_PKG_VERSION"));
const GEOCODING_API_HOST: &str = "geocoding-api.open-meteo.com";
const FORECASTING_API_HOST: &str = "api.open-meteo.com";

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

        match resp {
            MeteoResponse::Success(s) => Ok(s),
            MeteoResponse::Error { error: _, reason } => Err(Error::Meteo(reason)),
        }
    }

    /// Endpoint: `/search`
    pub async fn geocode(
        &self,
        name: &str,
        opt: Option<GeocodeOptions>,
    ) -> Result<Vec<GeocodeResult>> {
        let resp: GeocodeResponse = self
            .request(GEOCODING_API_HOST, "search", Some(&[("name", name)]), opt)
            .await?;

        Ok(resp.results)
    }

    /// Endpoint: `/forecast`
    pub async fn forecast_single(
        &self,
        latitude: f64,
        longitude: f64,
        opt: Option<ForecastOptions>,
    ) -> Result<ForecastResponse> {
        self.request(
            FORECASTING_API_HOST,
            "forecast",
            Some(&[("latitude", latitude), ("longitude", longitude)]),
            opt,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_london(client: &MeteoClient) -> GeocodeResult {
        let res = client
            .geocode("London, United Kingdom", None)
            .await
            .unwrap();

        res.get(0).unwrap().clone()
    }

    #[tokio::test]
    async fn geocode() {
        let client = MeteoClient::new(None).unwrap();
        let london = get_london(&client).await;

        assert_eq!(london.timezone, "Europe/London");
        assert_eq!(london.admin1, Some("England".to_string()));
        assert_eq!(london.country, "United Kingdom");
    }

    #[tokio::test]
    async fn forecast_single() {
        let client = MeteoClient::new(None).unwrap();
        let london = get_london(&client).await;

        client
            .forecast_single(
                london.latitude,
                london.longitude,
                Some(ForecastOptions {
                    hourly: Some(vec![HourlyVariable::Temperature2m]),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
    }
}
