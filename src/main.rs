use chrono::{DateTime, Local};
use iced::{
    Color, Element, Font, Length, Subscription,
    font::Weight,
    time,
    widget::{center, column, container, horizontal_space, row, stack, text},
};

use config::Config;

use background::{BackgroundKind, background};

use crate::icon::{icon, icon_button};

mod background;
mod config;
mod icon;

struct Fjordgard {
    config: Config,
    time: DateTime<Local>,
    background: BackgroundKind,
}

#[derive(Debug, Clone, Copy)]
enum MediaControl {
    Pause,
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(DateTime<Local>),
    Media(MediaControl),
    Settings,
}

impl Fjordgard {
    fn title(&self) -> String {
        String::from("Fjordgard")
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::Tick(time) => self.time = time,
            _ => {}
        }
    }

    fn view(&self) -> Element<Message> {
        let time_text = self.time.format(&self.config.time_format).to_string();

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

        let settings = icon_button("icons/settings.svg", Message::Settings);

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

    fn tick(&self) -> Subscription<Message> {
        time::every(time::Duration::from_secs(1)).map(|_| Message::Tick(Local::now()))
    }
}

impl Default for Fjordgard {
    fn default() -> Self {
        Self {
            config: Config::default(),
            time: Local::now(),
            background: BackgroundKind::Color(Color::from_rgb8(255, 255, 255)),
        }
    }
}

fn main() -> iced::Result {
    iced::application(Fjordgard::title, Fjordgard::update, Fjordgard::view)
        .subscription(Fjordgard::tick)
        .run()
}
