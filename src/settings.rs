use std::{cell::RefCell, rc::Rc};

use iced::{
    Element, Length, Task,
    widget::{
        button, column, combo_box, container, row, scrollable, text, text_input, vertical_space,
    },
    window,
};
use strum::VariantArray;

use crate::config::{BackgroundMode, Config};

#[derive(Debug, Clone, PartialEq, strum::Display, strum::VariantArray)]
enum WeatherLocation {
    Disabled,
    #[strum(to_string = "Location name")]
    LocationName,
    Coordinates,
}

pub struct Location {
    name: String,
    latitude: f64,
    longitude: f64,
}

pub struct Settings {
    pub id: window::Id,
    config: Rc<RefCell<Config>>,
    backgrounds: combo_box::State<BackgroundMode>,
    locations: combo_box::State<WeatherLocation>,

    time_format: String,
    background_mode: BackgroundMode,
    background: String,

    location: WeatherLocation,
    name: String,
    latitude: String,
    longitude: String,

    location_results: Vec<Location>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TimeFormat(String),
    BackgroundMode(BackgroundMode),
    Background(String),
    Location(WeatherLocation),
    Name(String),
    NameSubmitted,
    Latitude(String),
    Longitude(String),
    Save,
    FileSelector,
}

impl Settings {
    pub fn new(id: window::Id, config: Rc<RefCell<Config>>) -> Self {
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
            .map(|l| l.name.clone())
            .flatten()
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
            id,
            config,
            backgrounds: combo_box::State::new(BackgroundMode::VARIANTS.to_vec()),
            locations: combo_box::State::new(WeatherLocation::VARIANTS.to_vec()),

            time_format: original_config.time_format,
            background_mode: original_config.background_mode,
            background: original_config.background,

            location,
            latitude,
            longitude,
            name,

            location_results: vec![],
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Location(location) => {
                self.location = location;
                Task::none()
            }
            Message::BackgroundMode(mode) => {
                self.background = mode.default_background().to_string();
                self.background_mode = mode;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut background_mode_row =
            row![text(self.background_mode.edit_text()).width(Length::FillPortion(1))];

        if self.background_mode == BackgroundMode::Local {
            let text = if self.background == "" {
                "Select file..."
            } else {
                &self.background
            };

            background_mode_row = background_mode_row.push(
                button(text)
                    .on_press(Message::FileSelector)
                    .width(Length::FillPortion(2)),
            );
        } else {
            background_mode_row = background_mode_row.push(
                text_input(self.background_mode.default_background(), &self.background)
                    .on_input(Message::Background)
                    .width(Length::FillPortion(2)),
            );
        }

        let (latitude, longitude, name) = match self.location {
            WeatherLocation::Disabled => (None, None, None),
            WeatherLocation::LocationName => (None, None, Some(Message::Name)),
            WeatherLocation::Coordinates => {
                (Some(Message::Latitude), Some(Message::Longitude), None)
            }
        };

        let mut results = column![];

        for res in self.location_results.iter() {
            results = results.push(
                button(text(format!(
                    "{} ({}, {})",
                    res.name, res.latitude, res.longitude
                )))
                .style(button::text),
            )
        }

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
                        text("Weather Location").width(Length::FillPortion(1)),
                        combo_box(&self.locations, "", Some(&self.location), Message::Location)
                            .width(Length::FillPortion(2))
                    ],
                    row![
                        text("Latitude").width(Length::FillPortion(1)),
                        text_input("", &self.latitude)
                            .width(Length::FillPortion(2))
                            .on_input_maybe(latitude)
                    ],
                    row![
                        text("Longitude").width(Length::FillPortion(1)),
                        text_input("", &self.longitude)
                            .width(Length::FillPortion(2))
                            .on_input_maybe(longitude)
                    ],
                    row![
                        text("Location").width(Length::FillPortion(1)),
                        text_input("", &self.name)
                            .width(Length::FillPortion(2))
                            .on_input_maybe(name),
                    ],
                    scrollable(results)
                        .height(Length::Fixed(
                            64.0 * (self.location_results.len().clamp(0, 1) as f32)
                        ))
                        .width(Length::Fill),
                    button("Save").on_press(Message::Save)
                ]
                .spacing(10),
            )
            .padding(15),
        )
        .into()
    }
}
