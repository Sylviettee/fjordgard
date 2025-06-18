use chrono::{DateTime, Local};
use iced::{time, widget::text, Element, Subscription};

use config::Config;

mod config;

struct Fjordgard {
    config: Config,
    time: DateTime<Local>
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(DateTime<Local>)
}

impl Fjordgard {
    fn title(&self) -> String {
        String::from("Fjordgard")
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::Tick(time) => self.time = time,
        }
    }

    fn view(&self) -> Element<Message> {
        let time_text = self.time.format(&self.config.time_format);

        text(time_text.to_string())
            .size(20)
            .into()
    }

    fn tick(&self) -> Subscription<Message> {
        time::every(time::Duration::from_secs(1))
            .map(|_| Message::Tick(Local::now()))
    }
}

impl Default for Fjordgard {
    fn default() -> Self {
        Self {
            config: Config::default(),
            time: Local::now()
        }
    }
}

fn main() -> iced::Result {
    iced::application(Fjordgard::title, Fjordgard::update, Fjordgard::view)
        .subscription(Fjordgard::tick)
        .run()
}
