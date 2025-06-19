use std::{cell::RefCell, rc::Rc};

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
use config::{BackgroundMode, Config};
use icon::{icon, icon_button};
use strum::VariantArray;

mod background;
mod config;
mod icon;
mod settings;

struct Fjordgard {
    config: Rc<RefCell<Config>>,
    time: DateTime<Local>,
    background: BackgroundKind,

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

    SettingsOpened(window::Id),
    MainWindowOpened,
    WindowClosed(window::Id),

    Settings(settings::Message),
}

impl Fjordgard {
    fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings::default());

        (
            Self {
                config: Rc::new(RefCell::new(Config::default())),
                time: Local::now(),
                background: BackgroundKind::Color(Color::from_rgb8(255, 255, 255)),

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

                    self.settings_window = Some(settings::Settings::new(id, self.config.clone()));

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
            Message::Settings(msg) => {
                if let Some(settings) = &mut self.settings_window {
                    settings.update(msg).map(Message::Settings)
                } else {
                    Task::none()
                }
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
        let config = self.config.borrow();
        let dt = self.time.format(&config.time_format);
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
