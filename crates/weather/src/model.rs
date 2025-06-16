use std::{collections::HashMap, fmt::Display, hash::Hash, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use serde_with::DeserializeFromStr;
use strum::{Display, EnumString};

use crate::Error;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum MeteoResponse {
    Success(serde_json::Value),
    Error { reason: String },
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
pub struct GeocodeOptions {
    pub count: Option<usize>,
    pub language: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Location {
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

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GeocodeResponse {
    #[serde(default)]
    pub(crate) results: Vec<Location>,
}

#[derive(Display, EnumString, Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum HourlyVariable {
    #[strum(to_string = "temperature_2m")]
    Temperature2m,
    #[strum(to_string = "temperature_{0}hPa")]
    TemperaturePressureLevel(usize),
    #[strum(to_string = "relative_humidity_2m")]
    RelativeHumidity2m,
    #[strum(to_string = "relative_humidity_{0}hPa")]
    RelativeHumidityPressureLevel(usize),
    #[strum(to_string = "dew_point_2m")]
    DewPoint2m,
    #[strum(to_string = "dew_point_{0}hPa")]
    DewPointPressureLevel(usize),
    ApparentTemperature,
    PressureMsl,
    SurfacePressure,
    CloudCover,
    CloudCoverLow,
    CloudCoverMid,
    CloudCoverHigh,
    #[strum(to_string = "cloud_cover_{0}hPa")]
    CloudCoverPressureLevel(usize),
    #[strum(to_string = "wind_speed_10m")]
    WindSpeed10m,
    #[strum(to_string = "wind_speed_80m")]
    WindSpeed80m,
    #[strum(to_string = "wind_speed_120m")]
    WindSpeed120m,
    #[strum(to_string = "wind_speed_180m")]
    WindSpeed180m,
    #[strum(to_string = "wind_speed_{0}hPa")]
    WindSpeedPressureLevel(usize),
    #[strum(to_string = "wind_direction_10m")]
    WindDirection10m,
    #[strum(to_string = "wind_direction_80m")]
    WindDirection80m,
    #[strum(to_string = "wind_direction_120m")]
    WindDirection120m,
    #[strum(to_string = "wind_direction_180m")]
    WindDIrection180m,
    #[strum(to_string = "wind_direction_{0}hPa")]
    WindDirectionPressureLevel(usize),
    #[strum(to_string = "wind_gusts_10m")]
    WindGusts10m,
    ShortwaveRadiation,
    DirectRadiation,
    DirectNormalIrradiance,
    DiffuseRadiation,
    GlobalTiltedIrradiance,
    VapourPressureDeficit,
    Cape,
    Evapotranspiration,
    Et0FaoEvapotranspiration,
    Precipitation,
    Snowfall,
    PrecipitationProbability,
    Rain,
    Showers,
    WeatherCode,
    SnowDepth,
    FreezingLevelHeight,
    Visibility,
    #[strum(to_string = "soil_temperature_0cm")]
    SoilTemperature0cm,
    #[strum(to_string = "soil_temperature_6cm")]
    SoilTemperature6cm,
    #[strum(to_string = "soil_temperature_18cm")]
    SoilTemperature18cm,
    #[strum(to_string = "soil_temperature_54cm")]
    SoilTemperature54cm,
    #[strum(to_string = "soil_moisture_0_to_1cm")]
    SoilMoisture0To1cm,
    #[strum(to_string = "soil_moisture_1_to_3cm")]
    SoilMoisture1To3cm,
    #[strum(to_string = "soil_moisture_3_to_9cm")]
    SoilMoisture3To9cm,
    #[strum(to_string = "soil_moisture_9_to_27cm")]
    SoilMoisture9To27cm,
    #[strum(to_string = "soil_moisture_27_to_81cm")]
    SoilMoisture28To81cm,
    IsDay,
    #[strum(to_string = "geopotential_height_{0}hPa")]
    GeopotentialHeightPressureLevel(usize),
    /// NOTE: Not a valid variable, only found within `.hourly_units`
    Time,
}

impl<'de> Deserialize<'de> for HourlyVariable {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct HourlyVariableVisitor;

        impl<'de> Visitor<'de> for HourlyVariableVisitor {
            type Value = HourlyVariable;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid hourly variable")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match Self::Value::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        // temperature_{0}hPa
                        // relative_humidity_{0}hPa
                        // dew_point_{0}hPa
                        // cloud_cover_{0}hPa
                        // wind_speed_{0}hPa
                        // wind_direction_{0}hPa
                        // geopotential_height_{0}hPa

                        if !v.ends_with("hPa") {
                            return Err(serde::de::Error::custom(e));
                        }

                        let stripped = v.strip_suffix("hPa").unwrap();

                        let pos = stripped
                            .find(|c: char| c.is_ascii_digit())
                            .ok_or(serde::de::Error::custom(Error::InvalidPressureLevel))?;

                        let var = &stripped[..pos];
                        let num = stripped[pos..]
                            .parse::<usize>()
                            .map_err(serde::de::Error::custom)?;

                        let res = match var {
                            "temperature_" => HourlyVariable::TemperaturePressureLevel(num),
                            "relative_humidity_" => {
                                HourlyVariable::RelativeHumidityPressureLevel(num)
                            }
                            "dew_point_" => HourlyVariable::DewPointPressureLevel(num),
                            "cloud_cover_" => HourlyVariable::CloudCoverPressureLevel(num),
                            "wind_speed_" => HourlyVariable::WindSpeedPressureLevel(num),
                            "wind_direction_" => HourlyVariable::WindDirectionPressureLevel(num),
                            "geopotential_height_" => {
                                HourlyVariable::GeopotentialHeightPressureLevel(num)
                            }
                            _ => return Err(serde::de::Error::custom(Error::InvalidPressureLevel)),
                        };

                        Ok(res)
                    }
                }
            }
        }

        deserializer.deserialize_str(HourlyVariableVisitor)
    }
}

#[derive(Display, EnumString, Clone, Copy, Debug, Hash, PartialEq, Eq, DeserializeFromStr)]
#[strum(serialize_all = "snake_case")]
pub enum DailyVariable {
    #[strum(to_string = "temperature_2m_max")]
    Temperature2mMax,
    #[strum(to_string = "temperature_2m_mean")]
    Temperature2mMean,
    #[strum(to_string = "temperature_2m_min")]
    Temperature2mMin,
    ApparentTemperatureMax,
    ApparentTemperatureMean,
    ApparentTemperatureMin,
    PrecipitationSum,
    RainSum,
    ShowersSum,
    SnowfallSum,
    PrecipitationHours,
    PrecipitationProbabilityMax,
    PrecipitationProbabilityMean,
    PrecipitationProbabilityMin,
    WeatherCode,
    Sunrise,
    Sunset,
    SunshineDuration,
    DaylightDuration,
    #[strum(to_string = "wind_speed_10m_max")]
    WindSpeed10mMax,
    #[strum(to_string = "wind_gusts_10m_max")]
    WindGusts10mMax,
    #[strum(to_string = "wind_direction_10m_dominant")]
    WindDirection10mDominant,
    ShortwaveRadiationSum,
    Et0FaoEvapotranspiration,
    UvIndexMax,
    UvIndexClearSkyMax,
    /// NOTE: Not a valid variable, only found within `.daily_units`
    Time,
}

#[derive(Display, EnumString, Clone, Copy, Debug, Hash, PartialEq, Eq, DeserializeFromStr)]
#[strum(serialize_all = "snake_case")]
pub enum CurrentVariable {
    #[strum(to_string = "temperature_2m")]
    Temperature2m,
    #[strum(to_string = "relative_humidity_2m")]
    RelativeHumidity2m,
    #[strum(to_string = "dew_point_2m")]
    DewPoint2m,
    ApparentTemperature,
    ShortwaveRadiation,
    DirectRadiation,
    DirectNormalIrradiance,
    GlobalTiltedIrradiance,
    GlobalTiltedIrradianceInstant,
    DiffuseRadiation,
    SunshineDuration,
    LightningPotential,
    Precipitation,
    Snowfall,
    Rain,
    Showers,
    SnowfallHeight,
    FreezingLevelHeight,
    Cape,
    #[strum(to_string = "wind_speed_10m")]
    WindSpeed10m,
    #[strum(to_string = "wind_speed_80m")]
    WindSpeed80m,
    #[strum(to_string = "wind_direction_10m")]
    WindDirection10m,
    #[strum(to_string = "wind_direction_80m")]
    WindDirection80m,
    #[strum(to_string = "wind_gusts_10m")]
    WindGusts10m,
    Visibility,
    WeatherCode,
    /// NOTE: Not a valid variable, only found within `.current_units`
    Time,
    /// NOTE: Not a valid variable, only found within `.current_units`
    Interval,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

#[derive(Serialize)]
pub enum SpeedUnit {
    #[serde(rename = "kmh")]
    KilometersPerHour,
    #[serde(rename = "ms")]
    MetersPerSecond,
    #[serde(rename = "mph")]
    MilesPerHour,
    #[serde(rename = "kn")]
    Knots,
}

#[derive(Serialize)]
pub enum PrecipitationUnit {
    #[serde(rename = "mm")]
    Millimeter,
    #[serde(rename = "inch")]
    Inch,
}

#[derive(Serialize)]
#[serde(rename = "lowercase")]
pub enum TimeFormat {
    Iso8601,
    UnixTime,
}

#[derive(Serialize)]
#[serde(rename = "lowercase")]
pub enum CellSelection {
    Land,
    Sea,
    Nearest,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
pub struct ForecastOptions {
    pub elevation: Option<f64>,
    #[serde(serialize_with = "csv")]
    pub hourly: Option<Vec<HourlyVariable>>,
    #[serde(serialize_with = "csv")]
    pub daily: Option<Vec<DailyVariable>>,
    #[serde(serialize_with = "csv")]
    pub current: Option<Vec<CurrentVariable>>,
    pub temperature_unit: Option<TemperatureUnit>,
    pub wind_speed_unit: Option<SpeedUnit>,
    pub precipitation_unit: Option<PrecipitationUnit>,
    pub time_format: Option<TimeFormat>,
    pub timezone: Option<String>,
    pub past_days: Option<usize>,
    pub past_hours: Option<usize>,
    pub past_minutely_15: Option<usize>,
    pub forecast_days: Option<usize>,
    pub forecast_hours: Option<usize>,
    pub forecast_minutely_15: Option<usize>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub start_hour: Option<String>,
    pub end_hour: Option<String>,
    pub start_minutely_15: Option<String>,
    pub end_minutely_15: Option<String>,
    #[serde(serialize_with = "csv")]
    pub models: Option<Vec<String>>,
    pub cell_selection: Option<CellSelection>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HourlyData {
    pub time: Vec<String>,
    #[serde(flatten)]
    pub data: HashMap<HourlyVariable, Vec<f64>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DailyData {
    pub time: Vec<String>,
    #[serde(flatten)]
    pub data: HashMap<DailyVariable, Vec<f64>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CurrentData {
    pub time: String,
    pub interval: usize,
    #[serde(flatten)]
    pub data: HashMap<CurrentVariable, f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Forecast {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub utc_offset_seconds: isize,
    pub timezone: String,
    pub timezone_abbreviation: String,
    pub hourly: Option<HourlyData>,
    pub hourly_units: Option<HashMap<HourlyVariable, String>>,
    pub daily: Option<DailyData>,
    pub daily_units: Option<HashMap<DailyVariable, String>>,
    pub current: Option<CurrentData>,
    pub current_units: Option<HashMap<CurrentVariable, String>>,
}

fn csv<S: Serializer, T: Display>(list: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error> {
    if let Some(list) = list {
        let s: String = list
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");

        serializer.serialize_str(&s)
    } else {
        serializer.serialize_none()
    }
}
