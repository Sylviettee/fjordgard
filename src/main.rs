use std::{cell::RefCell, rc::Rc, sync::Arc};

use chrono::{
    DateTime, Local,
    format::{Item, StrftimeItems},
};
use fjordgard_weather::MeteoClient;
use iced::{
    Color, Element, Font, Length, Size, Subscription, Task,
    font::Weight,
    time,
    widget::{center, column, container, horizontal_space, row, stack, text},
    window,
};

use background::BackgroundHandle;
use config::Config;
use icon::{icon, icon_button};
use log::debug;

use crate::config::BackgroundMode;

mod background;
mod config;
mod icon;
mod settings;

struct Fjordgard {
    config: Rc<RefCell<Config>>,
    meteo: Arc<MeteoClient>,
    time: DateTime<Local>,
    background: BackgroundHandle,
    format_string: String,
    format_parsed: Vec<Item<'static>>,

    settings_window: Option<settings::Settings>,
    main_window: window::Id,
}

#[derive(Debug, Clone, Copy)]
enum MediaControl {
    Pause,
    Previous,
    Next,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(DateTime<Local>),
    Media(MediaControl),
    OpenSettings,

    SettingsOpened,
    MainWindowOpened,
    WindowClosed(window::Id),

    Settings(settings::Message),
    Background(background::Message),
}

impl Fjordgard {
    fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings::default());
        let config = Config::default();

        let format_string = config.time_format.clone();
        let format_parsed = StrftimeItems::new_lenient(&format_string)
            .parse_to_owned()
            .unwrap();

        let meteo = MeteoClient::new(None).unwrap();
        let (background, task) = BackgroundHandle::new(&config);

        (
            Self {
                config: Rc::new(RefCell::new(config)),
                meteo: Arc::new(meteo),
                time: Local::now(),
                background,
                format_string,
                format_parsed,

                settings_window: None,
                main_window: id,
            },
            Task::batch(vec![
                open.map(|_| Message::MainWindowOpened),
                task.map(Message::Background),
            ]),
        )
    }

    fn title(&self, window_id: window::Id) -> String {
        if window_id == self.main_window {
            String::from("Fjordgard")
        } else {
            String::from("Settings - Fjordgard")
        }
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tick(time) => {
                self.time = time;

                Task::none()
            }
            Message::OpenSettings => {
                if self.settings_window.is_none() {
                    let (_id, open) = window::open(window::Settings {
                        level: window::Level::AlwaysOnTop,
                        size: Size::new(350.0, 450.0),
                        ..Default::default()
                    });

                    self.settings_window = Some(settings::Settings::new(
                        self.config.clone(),
                        self.meteo.clone(),
                    ));

                    open.map(|_| Message::SettingsOpened)
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
            Message::Settings(settings::Message::Committed) => {
                let config = self.config.borrow();
                let config_format = &config.time_format;

                if &self.format_string != config_format {
                    self.format_string = config_format.clone();
                    self.format_parsed = StrftimeItems::new_lenient(config_format)
                        .parse_to_owned()
                        .unwrap();
                }

                self.background
                    .load_config(&config)
                    .map(Message::Background)
            }
            Message::Settings(msg) => {
                if let Some(settings) = &mut self.settings_window {
                    settings.update(msg).map(Message::Settings)
                } else {
                    Task::none()
                }
            }
            Message::Background(msg) => self.background.update(msg).map(Message::Background),
            Message::SettingsOpened => {
                debug!("settings window opened");
                Task::none()
            },
            Message::MainWindowOpened => {
                debug!("main window opened");
                Task::none()
            }
            _ => Task::none(),
        }
    }

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

    fn view_main(&self) -> Element<Message> {
        let mut bold = Font::DEFAULT;
        bold.weight = Weight::Bold;

        let time_text = self.time.format_with_items(self.format_parsed.iter());
        let time_widget = text(time_text.to_string())
            .size(100)
            .font(bold)
            .color(Color::WHITE)
            .width(Length::Fill)
            .center();

        let weather_widget = container(row![
            icon("icons/weather/not-available.svg"),
            horizontal_space().width(Length::Fixed(7.25)),
            text("Weather unknown").color(Color::WHITE)
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
        Subscription::batch(vec![
            time::every(time::Duration::from_secs(1)).map(|_| Message::Tick(Local::now())),
            window::close_events().map(Message::WindowClosed),
        ])
    }
}

fn main() -> iced::Result {
    env_logger::init();

    iced::daemon(Fjordgard::title, Fjordgard::update, Fjordgard::view)
        .subscription(Fjordgard::subscription)
        .run_with(Fjordgard::new)
}
