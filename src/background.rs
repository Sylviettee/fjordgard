use iced::{
    Color, ContentFit, Element, Length, Point, Renderer, Size, Task, Theme, mouse,
    widget::{canvas, container, image, stack, text},
};
use log::{debug, error};
use tokio::fs;

use crate::config::{BackgroundMode, Config};

struct Solid(Color);

impl<Message> canvas::Program<Message> for Solid {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(bounds.width, bounds.height),
            self.0,
        );

        vec![frame.into_geometry()]
    }
}

pub struct BackgroundHandle {
    pub mode: BackgroundMode,
    background: String,

    image_handle: Option<image::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundRead(Result<Vec<u8>, String>),
}

impl BackgroundHandle {
    pub fn new(config: &Config) -> (Self, Task<Message>) {
        let mut handle = Self {
            mode: config.background_mode,
            background: config.background.clone(),
            image_handle: None,
        };

        let task = handle.refresh();

        return (handle, task);
    }

    pub fn load_config(&mut self, config: &Config) -> Task<Message> {
        self.mode = config.background_mode;
        self.background = config.background.clone();

        self.refresh()
    }

    fn refresh(&mut self) -> Task<Message> {
        debug!("refreshing background (mode={}, background={})", self.mode, &self.background);

        match self.mode {
            BackgroundMode::Local => {
                let path = self.background.clone();

                Task::future(async move { fs::read(&path).await })
                    .map(|r| Message::BackgroundRead(r.map_err(|e| e.to_string())))
            }
            _ => Task::none(),
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::BackgroundRead(res) => match res {
                Err(e) => {
                    error!("failed to load image: {e}");
                    Task::none()
                },
                Ok(bytes) => {
                    self.image_handle = Some(image::Handle::from_bytes(bytes));
                    Task::none()
                }
            },
        }
    }

    fn solid<'a>(color: Color) -> Element<'a, Message> {
        canvas(Solid(color))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        match self.mode {
            BackgroundMode::Solid => {
                Self::solid(Color::parse(&self.background).unwrap_or(Color::BLACK))
            }
            _ => {
                if let Some(handle) = &self.image_handle {
                    let img = image(handle)
                        .content_fit(ContentFit::Cover)
                        .width(Length::Fill)
                        .height(Length::Fill);

                    if self.mode == BackgroundMode::Local {
                        img.into()
                    } else {
                        stack![
                            img,
                            // TODO; finish credits
                            container(text("Photo, John Doe, Unsplash"))
                                .align_left(Length::Fill)
                                .align_bottom(Length::Fill)
                                .padding(15)
                        ]
                        .into()
                    }
                } else {
                    Self::solid(Color::BLACK)
                }
            }
        }
    }
}
