use std::{cell::RefCell, rc::Rc, sync::Arc};

use chrono::{
    DateTime, Local,
    format::{Item, StrftimeItems},
};
use fjordgard_weather::{
    MeteoClient,
    model::{CurrentVariable, Forecast, ForecastOptions},
};
#[cfg(not(target_arch = "wasm32"))]
use iced::font::Weight;
use iced::{
    Color, Element, Font, Length, Size, Subscription, Task, time,
    widget::{center, column, container, horizontal_space, row, stack, text},
    window,
};

use background::BackgroundHandle;
use config::{BackgroundMode, Config};
use icon::{icon, icon_button};
use log::{debug, error};

mod background;
mod config;
mod icon;
mod settings;

pub struct Fjordgard {
    config: Rc<RefCell<Config>>,
    meteo: Arc<MeteoClient>,
    time: DateTime<Local>,
    background: BackgroundHandle,
    format_string: String,
    format_parsed: Vec<Item<'static>>,

    settings_window: Option<settings::Settings>,
    settings_id: Option<window::Id>,
    main_window: window::Id,
    main_window_size: Size,

    coordinate_pair: Option<(f64, f64)>,
    forecast_text: String,
    forecast_icon: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MediaControl {
    Pause,
    Previous,
    Next,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(DateTime<Local>),
    Media(MediaControl),
    OpenSettings,

    SettingsOpened(window::Id),
    MainWindowOpened,
    WindowClosed(window::Id),
    WindowResized((window::Id, Size)),

    Settings(settings::Message),
    Background(background::Message),

    RequestForecastUpdate,
    ForecastUpdate(Box<Result<Forecast, String>>),
}

#[cfg(target_arch = "wasm32")]
fn window_open(_settings: window::Settings) -> (window::Id, Task<window::Id>) {
    let id = window::Id::unique();

    (id, Task::done(id))
}

impl Fjordgard {
    fn new() -> (Self, Task<Message>) {
        let settings = window::Settings::default();
        let main_window_size = settings.size;

        #[cfg(not(target_arch = "wasm32"))]
        let (id, open) = window::open(settings);
        #[cfg(target_arch = "wasm32")]
        let (id, open) = window_open(settings);

        let config = Config::load().unwrap();

        let format_string = config.time_format.clone();
        let format_parsed = StrftimeItems::new_lenient(&format_string)
            .parse_to_owned()
            .unwrap();

        let meteo = MeteoClient::new(None).unwrap();
        let (background, task) = BackgroundHandle::new(&config, main_window_size);

        (
            Self {
                config: Rc::new(RefCell::new(config)),
                meteo: Arc::new(meteo),
                time: Local::now(),
                background,
                format_string,
                format_parsed,

                settings_window: None,
                settings_id: None,
                main_window: id,
                main_window_size,

                coordinate_pair: None,
                forecast_text: String::from("Weather unknown"),
                forecast_icon: String::from("icons/weather/100-0.svg"),
            },
            Task::batch([
                open.map(|_| Message::MainWindowOpened),
                task.map(Message::Background),
                Task::done(Message::RequestForecastUpdate),
            ]),
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn title(&self, window_id: window::Id) -> String {
        if window_id == self.main_window {
            String::from("Fjordgard")
        } else {
            String::from("Settings - Fjordgard")
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn title(&self) -> String {
        String::from("Fjordgard")
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tick(time) => {
                self.time = time;

                Task::none()
            }
            Message::Media(action) => match action {
                MediaControl::Next => {
                    Task::done(Message::Background(background::Message::RequestUnsplash(1)))
                }
                MediaControl::Previous => Task::done(Message::Background(
                    background::Message::RequestUnsplash(-1),
                )),
                MediaControl::Pause => {
                    Task::done(Message::Background(background::Message::PauseUnsplash))
                }
            },
            Message::OpenSettings => {
                if self.settings_window.is_none() {
                    #[cfg(not(target_arch = "wasm32"))]
                    let (_id, open) = window::open(window::Settings {
                        level: window::Level::AlwaysOnTop,
                        size: Size::new(350.0, 450.0),
                        ..Default::default()
                    });

                    #[cfg(target_arch = "wasm32")]
                    let (_id, open) = window_open(window::Settings::default());

                    self.settings_window = Some(settings::Settings::new(
                        self.config.clone(),
                        self.meteo.clone(),
                    ));

                    open.map(Message::SettingsOpened)
                } else {
                    Task::none()
                }
            }
            Message::WindowClosed(id) => {
                if self.main_window == id {
                    iced::exit()
                } else {
                    self.settings_window = None;
                    Task::none()
                }
            }
            Message::WindowResized((id, size)) => {
                if self.main_window != id {
                    return Task::none();
                }

                self.main_window_size = size;

                Task::none()
            }
            Message::Settings(settings::Message::Committed) => {
                let config = self.config.borrow();
                let config_format = &config.time_format;

                if &self.format_string != config_format {
                    self.format_string = config_format.clone();
                    self.format_parsed = StrftimeItems::new_lenient(config_format)
                        .parse_to_owned()
                        .unwrap();
                }

                let background_task = self
                    .background
                    .load_config(&config, self.main_window_size)
                    .map(Message::Background);

                let new_pair = config.location.as_ref().map(|l| (l.latitude, l.longitude));

                if new_pair != self.coordinate_pair {
                    self.coordinate_pair = new_pair;
                    Task::batch([background_task, Task::done(Message::RequestForecastUpdate)])
                } else {
                    background_task
                }
            }
            #[cfg(target_arch = "wasm32")]
            Message::Settings(settings::Message::ToBackground(msg)) => {
                Task::done(Message::Background(msg))
            }
            Message::Settings(settings::Message::CloseSettings) => {
                #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
                if let Some(id) = self.settings_id {
                    self.settings_id = None;
                    self.settings_window = None;

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        window::close(id)
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::Settings(msg) => {
                if let Some(settings) = &mut self.settings_window {
                    settings.update(msg).map(Message::Settings)
                } else {
                    Task::none()
                }
            }
            Message::Background(msg) => self.background.update(msg).map(Message::Background),
            Message::SettingsOpened(id) => {
                debug!("settings window opened");
                self.settings_id = Some(id);
                Task::none()
            }
            Message::MainWindowOpened => {
                debug!("main window opened");
                Task::none()
            }
            Message::RequestForecastUpdate => {
                let config = self.config.borrow();
                if let Some(location) = &config.location {
                    let meteo = self.meteo.clone();
                    let (latitude, longitude) = (location.latitude, location.longitude);

                    Task::future(async move {
                        meteo
                            .forecast_single(
                                latitude,
                                longitude,
                                Some(ForecastOptions {
                                    current: Some(vec![
                                        CurrentVariable::Temperature2m,
                                        CurrentVariable::IsDay,
                                        CurrentVariable::WeatherCode,
                                    ]),
                                    ..Default::default()
                                }),
                            )
                            .await
                    })
                    .map(|r| Message::ForecastUpdate(Box::new(r.map_err(|e| e.to_string()))))
                } else {
                    self.forecast_text = String::from("Weather unknown");
                    self.forecast_icon = String::from("icons/weather/100-0.svg");

                    Task::none()
                }
            }
            Message::ForecastUpdate(res) => match *res {
                Err(e) => {
                    error!("failed to load forecast: {e}");
                    Task::none()
                }
                Ok(forecast) => {
                    let forecast = || -> Option<(String, String)> {
                        let current = forecast.current?;
                        let units = forecast.current_units?;

                        let temperature = current.data.get(&CurrentVariable::Temperature2m)?;
                        let temperature_units = units.get(&CurrentVariable::Temperature2m)?;

                        let is_day = *current.data.get(&CurrentVariable::IsDay)? as u64;
                        let weather_code = *current.data.get(&CurrentVariable::WeatherCode)? as u64;

                        let condition_text = match weather_code {
                            0 => {
                                if is_day == 0 {
                                    "Clear"
                                } else {
                                    "Sunny"
                                }
                            }
                            1 => {
                                if is_day == 0 {
                                    "Mainly clear"
                                } else {
                                    "Mainly sunny"
                                }
                            }
                            2 => "Partly cloudy",
                            3 => "Overcast",
                            45 => "Foggy",
                            48 => "Rime fog",
                            51 => "Light drizzle",
                            53 => "Drizzle",
                            55 => "Heavy drizzle",
                            56 => "Light freezing drizzle",
                            57 => "Freezing drizzle",
                            61 => "Light rain",
                            63 => "Rain",
                            65 => "Heavy rain",
                            66 => "Light freezing rain",
                            67 => "Freezing rain",
                            71 => "Light snow",
                            73 => "Snow",
                            75 => "Heavy snow",
                            77 => "Snow grains",
                            80 => "Light showers",
                            81 => "Showers",
                            82 => "Heavy showers",
                            85 => "Light snow showers",
                            86 => "Snow showers",
                            95 => "Thunderstorm",
                            96 => "Light thunderstorm with hail",
                            99 => "Thunderstorm with hail",
                            _ => "Unknown",
                        };

                        let icon_condition = match weather_code {
                            0 => 0,
                            1 => 1,
                            2 => 2,
                            3 => 3,
                            45 | 48 => 45,
                            51 | 53 | 55 | 56 | 57 => 51,
                            61 | 63 | 65 | 66 | 67 => 61,
                            71 | 73 | 75 => 71,
                            77 => 77,
                            80 | 81 | 82 | 85 | 86 => 80,
                            95 => 95,
                            96 | 99 => 96,
                            _ => 100,
                        };

                        Some((
                            format!("{temperature}{temperature_units} {condition_text}"),
                            format!("icons/weather/{icon_condition}-{is_day}.svg"),
                        ))
                    };

                    if let Some((forecast_text, forecast_icon)) = forecast() {
                        self.forecast_text = forecast_text;
                        self.forecast_icon = forecast_icon;
                    }

                    Task::none()
                }
            },
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn view(&self, window_id: window::Id) -> Element<Message> {
        if self.main_window == window_id {
            self.view_main()
        } else {
            self.settings_window
                .as_ref()
                .expect("settings window")
                .view()
                .map(Message::Settings)
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn view(&self) -> Element<Message> {
        if let Some(settings) = &self.settings_window {
            settings.view().map(Message::Settings)
        } else {
            self.view_main()
        }
    }

    fn view_main(&self) -> Element<Message> {
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut bold = Font::DEFAULT;
        #[cfg(not(target_arch = "wasm32"))]
        {
            bold.weight = Weight::Bold;
        }

        let time_text = self.time.format_with_items(self.format_parsed.iter());
        let time_widget = text(time_text.to_string())
            .size(200)
            .font(bold)
            .color(Color::WHITE)
            .width(Length::Fill)
            .center();

        let weather_widget = container(row![
            icon(&self.forecast_icon)
                .height(Length::Fixed(32.0))
                .width(Length::Fixed(32.0)),
            horizontal_space().width(Length::Fixed(7.25)),
            text(&self.forecast_text).color(Color::WHITE).size(25)
        ])
        .center_x(Length::Fill);

        let settings = icon_button("icons/settings.svg", Message::OpenSettings);

        let mut main_column = column![settings, center(column![time_widget, weather_widget])];

        if self.background.mode == BackgroundMode::Unsplash {
            main_column = main_column.push(
                container(
                    row![
                        icon_button("icons/previous.svg", Message::Media(MediaControl::Previous)),
                        icon_button("icons/pause.svg", Message::Media(MediaControl::Pause)),
                        icon_button("icons/next.svg", Message::Media(MediaControl::Next)),
                    ]
                    .spacing(5),
                )
                .center_x(Length::Fill),
            )
        }

        stack![
            self.background.view().map(Message::Background),
            container(main_column).padding(15)
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(time::Duration::from_secs(1)).map(|_| Message::Tick(Local::now())),
            time::every(time::Duration::from_secs(60 * 15)).map(|_| Message::RequestForecastUpdate),
            time::every(time::Duration::from_secs(60 * 15))
                .map(|_| Message::Background(background::Message::RequestUnsplash(1))),
            window::close_events().map(Message::WindowClosed),
            window::resize_events().map(Message::WindowResized),
        ])
    }
}

fn main() -> iced::Result {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        iced::daemon(Fjordgard::title, Fjordgard::update, Fjordgard::view)
            .subscription(Fjordgard::subscription)
            .run_with(Fjordgard::new)
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).unwrap();

        iced::application(Fjordgard::title, Fjordgard::update, Fjordgard::view)
            .subscription(Fjordgard::subscription)
            .run_with(Fjordgard::new)
    }
}
