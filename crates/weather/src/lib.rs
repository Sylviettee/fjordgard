use std::fmt::Debug;

use reqwest::Client;

pub use error::Error;
use error::Result;
use model::*;
use serde::{Serialize, de::DeserializeOwned};

mod error;
pub mod model;

#[cfg(not(target_arch = "wasm32"))]
const USER_AGENT: &str = concat!("fjordgard/", env!("CARGO_PKG_VERSION"));
const GEOCODING_API_HOST: &str = "geocoding-api.open-meteo.com";
const FORECASTING_API_HOST: &str = "api.open-meteo.com";

pub struct MeteoClient {
    api_key: Option<String>,
    client: Client,
}

impl MeteoClient {
    pub fn new(api_key: Option<&str>) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        #[cfg(target_arch = "wasm32")]
        let client = Client::new();

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

        let resp: MeteoResponse = req.send().await?.json().await?;

        match resp {
            MeteoResponse::Error { reason } => Err(Error::Meteo(reason)),
            MeteoResponse::Success(v) => match serde_json::from_value(v) {
                Ok(o) => Ok(o),
                Err(e) => Err(Error::SerdeJson(e)),
            },
        }
    }

    /// Endpoint: `/search`
    pub async fn geocode(&self, name: &str, opt: Option<GeocodeOptions>) -> Result<Vec<Location>> {
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
    ) -> Result<Forecast> {
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

    async fn get_london(client: &MeteoClient) -> Location {
        let res = client
            .geocode("London, United Kingdom", None)
            .await
            .unwrap();

        res.first().unwrap().clone()
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
                    current: Some(vec![CurrentVariable::Temperature2m]),
                    daily: Some(vec![DailyVariable::Temperature2mMean]),
                    hourly: Some(vec![
                        HourlyVariable::Temperature2m,
                        HourlyVariable::TemperaturePressureLevel(1000),
                    ]),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
    }
}
