use std::{cell::RefCell, rc::Rc, sync::Arc};

use fjordgard_weather::{MeteoClient, model::Location};
use iced::{
    Background, Border, Color, Element, Length, Task, Theme,
    widget::{button, column, combo_box, container, row, scrollable, text, text_input, tooltip},
};
use log::error;
#[cfg(not(target_arch = "wasm32"))]
use rfd::{AsyncFileDialog, FileHandle};
use strum::VariantArray;

use crate::config::{self, BackgroundMode, Config};

#[derive(Debug, Clone, PartialEq, strum::Display, strum::VariantArray)]
pub enum WeatherLocation {
    Disabled,
    #[strum(to_string = "Location name")]
    LocationName,
    Coordinates,
}

#[derive(Debug, Clone)]
pub struct LocationRow {
    name: String,
    latitude: f64,
    longitude: f64,
}

pub struct Settings {
    config: Rc<RefCell<Config>>,
    meteo: Arc<MeteoClient>,
    backgrounds: combo_box::State<BackgroundMode>,
    locations: combo_box::State<WeatherLocation>,
    #[cfg(not(target_arch = "wasm32"))]
    file_selector_open: bool,

    time_format: String,
    background_mode: BackgroundMode,
    background: String,
    unsplash_key: String,

    location: WeatherLocation,
    name: String,
    latitude: String,
    longitude: String,

    location_results: Vec<LocationRow>,
    location_fetch_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TimeFormat(String),
    BackgroundMode(BackgroundMode),
    Background(String),
    UnsplashKey(String),
    Location(WeatherLocation),
    Name(String),
    NameSubmitted,
    Geocode(Result<Vec<Location>, String>),
    LocationSelected(LocationRow),
    Latitude(String),
    Longitude(String),
    #[cfg(not(target_arch = "wasm32"))]
    FileSelector,
    #[cfg(not(target_arch = "wasm32"))]
    FileSelected(Option<FileHandle>),
    Save,
    CloseSettings,

    Committed,
    Saved(Result<(), String>),

    #[cfg(target_arch = "wasm32")]
    ToBackground(crate::background::Message),
}

impl Settings {
    pub fn new(config: Rc<RefCell<Config>>, meteo: Arc<MeteoClient>) -> Self {
        let original_config = config.borrow().clone();
        let location = original_config.location;

        let latitude = location
            .as_ref()
            .map(|l| l.latitude.to_string())
            .unwrap_or_default();
        let longitude = location
            .as_ref()
            .map(|l| l.longitude.to_string())
            .unwrap_or_default();
        let name = location
            .as_ref()
            .and_then(|l| l.name.clone())
            .unwrap_or_default();
        let location = location
            .as_ref()
            .map(|l| {
                l.name
                    .as_ref()
                    .map(|_| WeatherLocation::LocationName)
                    .unwrap_or(WeatherLocation::Coordinates)
            })
            .unwrap_or(WeatherLocation::Disabled);

        Self {
            config,
            meteo,
            backgrounds: combo_box::State::new(BackgroundMode::VARIANTS.to_vec()),
            locations: combo_box::State::new(WeatherLocation::VARIANTS.to_vec()),
            #[cfg(not(target_arch = "wasm32"))]
            file_selector_open: false,

            time_format: original_config.time_format,
            background_mode: original_config.background_mode,
            background: original_config.background,
            unsplash_key: original_config.unsplash_key.unwrap_or_default(),

            location,
            latitude,
            longitude,
            name,

            location_results: vec![],
            location_fetch_error: None,
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::TimeFormat(format) => {
                self.time_format = format;
                Task::none()
            }
            Message::BackgroundMode(mode) => {
                self.background = mode.default_background().to_string();
                self.background_mode = mode;
                Task::none()
            }
            Message::Background(background) => {
                self.background = background;
                Task::none()
            }
            Message::UnsplashKey(key) => {
                self.unsplash_key = key;
                Task::none()
            }
            Message::Location(location) => {
                self.location = location;
                Task::none()
            }
            Message::Name(name) => {
                self.name = name;
                Task::none()
            }
            Message::NameSubmitted => {
                self.location_fetch_error = None;
                let meteo = self.meteo.clone();
                let name = self.name.clone();

                Task::future(async move { meteo.geocode(&name, None).await })
                    .map(|r| Message::Geocode(r.map_err(|e| e.to_string())))
            }
            Message::Geocode(locations) => {
                match locations {
                    Err(e) => {
                        error!("failed to fetch geocode: {e}");
                        self.location_fetch_error = Some(e);
                    }
                    Ok(res) => {
                        self.location_results = res
                            .iter()
                            .map(|l| {
                                let level1 = if let Some(admin1) = &l.admin1 {
                                    format!(", {admin1}")
                                } else {
                                    String::new()
                                };

                                LocationRow {
                                    name: format!("{}{level1}, {}", l.name, l.country),
                                    latitude: l.latitude,
                                    longitude: l.longitude,
                                }
                            })
                            .collect()
                    }
                };

                Task::none()
            }
            Message::LocationSelected(loc) => {
                self.name = loc.name;
                self.latitude = loc.latitude.to_string();
                self.longitude = loc.longitude.to_string();

                Task::none()
            }
            Message::Latitude(latitude) => {
                self.latitude = latitude;
                Task::none()
            }
            Message::Longitude(longitude) => {
                self.longitude = longitude;
                Task::none()
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::FileSelector => {
                if self.file_selector_open {
                    return Task::none();
                }

                self.file_selector_open = true;

                let file_task = AsyncFileDialog::new()
                    .add_filter("image", &["png", "jpeg", "jpg"])
                    .pick_file();

                Task::future(file_task).map(Message::FileSelected)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::FileSelected(file) => {
                self.file_selector_open = false;

                if let Some(file) = file {
                    self.background = file.path().to_string_lossy().to_string();
                }

                Task::none()
            }
            Message::Save => {
                let mut config = self.config.borrow_mut();

                config.time_format = self.time_format.clone();
                config.background_mode = self.background_mode;
                config.background = self.background.clone();
                config.unsplash_key = if self.unsplash_key.is_empty() {
                    None
                } else {
                    Some(self.unsplash_key.clone())
                };

                match self.location {
                    WeatherLocation::Disabled => config.location = None,
                    _ => {
                        config.location = Some(config::Location {
                            // this *should* be safe if we're at this point
                            longitude: self.longitude.parse().unwrap(),
                            latitude: self.latitude.parse().unwrap(),
                            name: if self.location == WeatherLocation::LocationName {
                                Some(self.name.clone())
                            } else {
                                None
                            },
                        })
                    }
                }

                let cloned = config.clone();

                Task::batch([
                    Task::done(Message::Committed),
                    Task::future(async move { cloned.save().await })
                        .map(|r| Message::Saved(r.map_err(|e| e.to_string()))),
                ])
            }
            Message::Saved(res) => match res {
                Err(e) => {
                    error!("failed to save config: {e}");
                    Task::none()
                }
                Ok(()) => Task::none(),
            },
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let (latitude, longitude, name) = match self.location {
            WeatherLocation::Disabled => (None, None, None),
            WeatherLocation::LocationName => (None, None, Some(Message::Name)),
            WeatherLocation::Coordinates => {
                (Some(Message::Latitude), Some(Message::Longitude), None)
            }
        };

        let mut save_message = Some(Message::Save);

        let color_style = if (self.background_mode == BackgroundMode::Solid
            && Color::parse(&self.background).is_none())
            || (self.background_mode == BackgroundMode::Unsplash && self.background.is_empty())
        {
            save_message = None;
            text_input_error
        } else {
            text_input::default
        };

        let unsplash_style =
            if self.background_mode == BackgroundMode::Unsplash && self.unsplash_key.is_empty() {
                save_message = None;
                text_input_error
            } else {
                text_input::default
            };

        let unsplash_key = if self.background_mode == BackgroundMode::Unsplash {
            Some(Message::UnsplashKey)
        } else {
            None
        };

        let latitude_style = if self.latitude.parse::<f64>().is_err()
            && matches!(
                self.location,
                WeatherLocation::LocationName | WeatherLocation::Coordinates
            ) {
            save_message = None;
            text_input_error
        } else {
            text_input::default
        };

        let longitude_style = if self.longitude.parse::<f64>().is_err()
            && matches!(
                self.location,
                WeatherLocation::LocationName | WeatherLocation::Coordinates
            ) {
            save_message = None;
            text_input_error
        } else {
            text_input::default
        };

        let mut background_mode_row =
            row![text(self.background_mode.edit_text()).width(Length::FillPortion(1))];

        match self.background_mode {
            #[cfg(not(target_arch = "wasm32"))]
            BackgroundMode::Local => {
                let text = if self.background.is_empty() {
                    save_message = None;
                    "Select file..."
                } else {
                    &self.background
                };

                background_mode_row = background_mode_row.push(
                    button(text)
                        .on_press(Message::FileSelector)
                        .width(Length::FillPortion(2)),
                );
            }
            _ => {
                background_mode_row = background_mode_row.push(
                    text_input(self.background_mode.default_background(), &self.background)
                        .on_input(Message::Background)
                        .width(Length::FillPortion(2))
                        .style(color_style),
                );
            }
        }

        let mut results = column![];

        for res in self.location_results.iter() {
            results = results.push(
                button(text(format!(
                    "{} ({}, {})",
                    res.name, res.latitude, res.longitude
                )))
                .style(button::text)
                .on_press_with(|| Message::LocationSelected(res.clone())),
            )
        }

        let location_style = if self.location_fetch_error.is_some() {
            save_message = None;
            text_input_error
        } else {
            text_input::default
        };

        let mut location_row: Element<Message> = row![
            text("Location").width(Length::FillPortion(1)),
            text_input("", &self.name)
                .width(Length::FillPortion(2))
                .on_input_maybe(name)
                .on_submit(Message::NameSubmitted)
                .style(location_style)
        ]
        .into();

        if let Some(err) = &self.location_fetch_error {
            location_row = tooltip(
                location_row,
                container(err.as_ref())
                    .padding(5)
                    .style(container::rounded_box),
                tooltip::Position::Top,
            )
            .into()
        };

        scrollable(
            container(
                column![
                    row![
                        text("Time format").width(Length::FillPortion(1)),
                        text_input("", &self.time_format)
                            .width(Length::FillPortion(2))
                            .on_input(Message::TimeFormat)
                    ],
                    row![
                        text("Background mode").width(Length::FillPortion(1)),
                        combo_box(
                            &self.backgrounds,
                            "",
                            Some(&self.background_mode),
                            Message::BackgroundMode
                        )
                        .width(Length::FillPortion(2))
                    ],
                    background_mode_row,
                    row![
                        text("Unsplash API Key").width(Length::FillPortion(1)),
                        text_input("", &self.unsplash_key)
                            .width(Length::FillPortion(2))
                            .on_input_maybe(unsplash_key)
                            .style(unsplash_style)
                    ],
                    row![
                        text("Weather Location").width(Length::FillPortion(1)),
                        combo_box(&self.locations, "", Some(&self.location), Message::Location)
                            .width(Length::FillPortion(2))
                    ],
                    row![
                        text("Latitude").width(Length::FillPortion(1)),
                        text_input("", &self.latitude)
                            .width(Length::FillPortion(2))
                            .on_input_maybe(latitude)
                            .style(latitude_style)
                    ],
                    row![
                        text("Longitude").width(Length::FillPortion(1)),
                        text_input("", &self.longitude)
                            .width(Length::FillPortion(2))
                            .on_input_maybe(longitude)
                            .style(longitude_style)
                    ],
                    location_row,
                    scrollable(results)
                        .height(Length::Fixed(
                            64.0 * (self.location_results.len().clamp(0, 1) as f32)
                        ))
                        .width(Length::Fill),
                    row![
                        button("Save").on_press_maybe(save_message),
                        button("Close").on_press(Message::CloseSettings),
                    ]
                    .spacing(5)
                ]
                .spacing(10),
            )
            .padding(15),
        )
        .into()
    }
}

fn text_input_error(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();

    let active = text_input::Style {
        background: Background::Color(palette.danger.weak.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.danger.strong.color,
        },
        icon: palette.danger.weak.text,
        placeholder: palette.danger.strong.color,
        value: palette.danger.weak.text,
        selection: palette.danger.strong.color,
    };

    match status {
        text_input::Status::Active => active,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: palette.danger.base.text,
                ..active.border
            },
            ..active
        },
        text_input::Status::Focused => text_input::Style {
            border: Border {
                color: palette.background.strong.color,
                ..active.border
            },
            ..active
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(palette.danger.weak.color),
            value: active.placeholder,
            ..active
        },
    }
}
