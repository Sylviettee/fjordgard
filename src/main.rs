use fjordgard_weather::{
    Error, MeteoClient,
    model::{CurrentVariable, ForecastOptions, GeocodeOptions},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    if let Some(location) = std::env::args().nth(1) {
        let client = MeteoClient::new(None)?;

        let locations = client
            .geocode(
                &location,
                Some(GeocodeOptions {
                    count: Some(1),
                    ..Default::default()
                }),
            )
            .await?;

        match locations.get(0) {
            None => eprintln!("location not found"),
            Some(loc) => {
                println!(
                    "{}{}, {} ({}, {})",
                    loc.name,
                    loc.admin1
                        .as_ref()
                        .map(|s| format!(", {s}"))
                        .unwrap_or_default(),
                    loc.country,
                    loc.latitude,
                    loc.longitude
                );

                let forecast = client
                    .forecast_single(
                        loc.latitude,
                        loc.longitude,
                        Some(ForecastOptions {
                            current: Some(vec![
                                CurrentVariable::Temperature2m,
                                CurrentVariable::ApparentTemperature,
                                CurrentVariable::RelativeHumidity2m,
                                CurrentVariable::WindSpeed10m,
                                CurrentVariable::WindDirection10m,
                            ]),
                            ..Default::default()
                        }),
                    )
                    .await?;
                let current = forecast.current.unwrap().data;
                let current_units = forecast.current_units.unwrap();

                println!(
                    "It is currently {}{} (feels like {}{}) with {}{} humidity",
                    current.get(&CurrentVariable::Temperature2m).unwrap(),
                    current_units.get(&CurrentVariable::Temperature2m).unwrap(),
                    current.get(&CurrentVariable::ApparentTemperature).unwrap(),
                    current_units
                        .get(&CurrentVariable::ApparentTemperature)
                        .unwrap(),
                    current.get(&CurrentVariable::RelativeHumidity2m).unwrap(),
                    current_units
                        .get(&CurrentVariable::RelativeHumidity2m)
                        .unwrap(),
                );
                println!(
                    "The wind speed is {}{} pointing {}{}",
                    current.get(&CurrentVariable::WindSpeed10m).unwrap(),
                    current_units.get(&CurrentVariable::WindSpeed10m).unwrap(),
                    current.get(&CurrentVariable::WindDirection10m).unwrap(),
                    current_units
                        .get(&CurrentVariable::WindDirection10m)
                        .unwrap(),
                );
            }
        }
    } else {
        eprintln!("location not specified")
    }

    Ok(())
}
