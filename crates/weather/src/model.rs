use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(crate) struct MeteoResponse<T> {
    pub(crate) results: Option<T>,
    pub(crate) error: Option<bool>,
    pub(crate) reason: Option<String>,
}

#[derive(Serialize)]
#[serde(default)]
pub struct GeocodeOptions {
    pub count: Option<usize>,
    pub language: Option<String>,
    pub api_key: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GeocodeResponse {
    pub id: usize,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub timezone: String,
    pub feature_code: String,
    pub country_code: String,
    pub country: String,
    pub country_id: usize,
    #[serde(default)]
    pub population: Option<usize>,
    #[serde(default)]
    pub postcodes: Vec<String>,
    #[serde(default)]
    pub admin1: Option<String>,
    #[serde(default)]
    pub admin2: Option<String>,
    #[serde(default)]
    pub admin3: Option<String>,
    #[serde(default)]
    pub admin4: Option<String>,
    #[serde(default)]
    pub admin1_id: Option<usize>,
    #[serde(default)]
    pub admin2_id: Option<usize>,
    #[serde(default)]
    pub admin3_id: Option<usize>,
    #[serde(default)]
    pub admin4_id: Option<usize>,
}
