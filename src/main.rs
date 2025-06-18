use chrono::{DateTime, Local};
use iced::{
    Color, Element, Font, Length, Size, Subscription, Task,
    font::Weight,
    time,
    widget::{
        button, center, column, combo_box, container, horizontal_space, row, stack, text,
        text_input,
    },
    window,
};

use background::{BackgroundKind, background};
use config::Config;
use icon::{icon, icon_button};
use strum::VariantArray;

use crate::config::BackgroundMode;

mod background;
mod config;
mod icon;

struct Fjordgard {
    config: Config,
    time: DateTime<Local>,
    background: BackgroundKind,
    backgrounds: combo_box::State<BackgroundMode>,

    settings_window: Option<window::Id>,
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

    SettingsOpened(window::Id),
    MainWindowOpened,
    WindowClosed(window::Id),

    ConfigBackgroundMode(BackgroundMode),
    ConfigTimeFormat(String),
}

impl Fjordgard {
    fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings::default());

        (
            Self {
                config: Config::default(),
                time: Local::now(),
                background: BackgroundKind::Color(Color::from_rgb8(255, 255, 255)),
                backgrounds: combo_box::State::new(BackgroundMode::VARIANTS.to_vec()),

                settings_window: None,
                main_window: id,
            },
            open.map(|_| Message::MainWindowOpened),
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
                    let (id, open) = window::open(window::Settings {
                        level: window::Level::AlwaysOnTop,
                        size: Size::new(350.0, 450.0),
                        ..Default::default()
                    });

                    self.settings_window = Some(id);

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
            Message::ConfigBackgroundMode(mode) => {
                self.config.background = mode.default_background().to_string();
                self.config.background_mode = mode;
                Task::none()
            }
            Message::ConfigTimeFormat(format) => {
                self.config.time_format = format;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        if self.main_window == window_id {
            self.view_main()
        } else {
            self.view_settings()
        }
    }

    fn view_main(&self) -> Element<Message> {
        let dt = self.time.format(&self.config.time_format);
        let mut time_text = String::new();

        if let Err(_) = dt.write_to(&mut time_text) {
            time_text = String::from("Invalid time format")
        }

        let mut bold = Font::DEFAULT;
        bold.weight = Weight::Bold;

        let time_widget = text(time_text)
            .size(100)
            .font(bold)
            .width(Length::Fill)
            .center();

        let weather_widget = container(row![
            icon("icons/weather/not-available.svg"),
            horizontal_space().width(Length::Fixed(7.25)),
            text("Weather unknown")
        ])
        .center_x(Length::Fill);

        let control = container(
            row![
                icon_button("icons/previous.svg", Message::Media(MediaControl::Previous)),
                icon_button("icons/pause.svg", Message::Media(MediaControl::Pause)),
                icon_button("icons/next.svg", Message::Media(MediaControl::Next)),
            ]
            .spacing(5),
        )
        .center_x(Length::Fill);

        let settings = icon_button("icons/settings.svg", Message::OpenSettings);

        stack![
            background(&self.background),
            container(column![
                settings,
                center(column![time_widget, weather_widget]),
                control
            ])
            .padding(15)
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    fn view_settings(&self) -> Element<Message> {
        let placeholder = Config::default();
        let config = &self.config;

        let mut background_mode_row =
            row![text(config.background_mode.edit_text()).width(Length::FillPortion(1)),];

        if config.background_mode == BackgroundMode::Local {
            let text = if config.background == "" {
                "Select file..."
            } else {
                &config.background
            };

            background_mode_row =
                background_mode_row.push(button(text).width(Length::FillPortion(2)));
        } else {
            background_mode_row = background_mode_row.push(
                text_input(
                    config.background_mode.default_background(),
                    &config.background,
                )
                .width(Length::FillPortion(2)),
            );
        }

        container(
            column![
                row![
                    text("Time format").width(Length::FillPortion(1)),
                    text_input(&placeholder.time_format, &config.time_format)
                        .width(Length::FillPortion(2))
                        .on_input(Message::ConfigTimeFormat)
                ],
                row![
                    text("Background mode").width(Length::FillPortion(1)),
                    combo_box(
                        &self.backgrounds,
                        "",
                        Some(&placeholder.background_mode),
                        Message::ConfigBackgroundMode
                    )
                    .width(Length::FillPortion(2))
                ],
                background_mode_row
            ]
            .spacing(10),
        )
        .padding(15)
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
    iced::daemon(Fjordgard::title, Fjordgard::update, Fjordgard::view)
        .subscription(Fjordgard::subscription)
        .run_with(Fjordgard::new)
}
