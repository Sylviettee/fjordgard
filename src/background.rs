use fjordgard_unsplash::{
    UnsplashClient,
    model::{Collection, CollectionPhotos, CollectionPhotosOptions, Format, PhotoFetchOptions},
};
use iced::{
    Color, ContentFit, Element, Length, Size, Task,
    widget::{button, container, image, row, stack, text},
};
use log::{debug, error};

use crate::config::{BackgroundMode, Config};

pub struct UnsplashState {
    collection: String,
    current: usize,
    total: usize,
    paused: bool,

    current_page_photos: Option<CollectionPhotos>,
    current_page: usize,
}

pub struct BackgroundHandle {
    pub mode: BackgroundMode,
    background: String,
    size: Size,

    image_handle: Option<image::Handle>,

    unsplash_key: Option<String>,
    unsplash_client: Option<UnsplashClient>,
    unsplash_state: Option<UnsplashState>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundRead(Result<Vec<u8>, String>),
    UnsplashCollection(Box<Result<Collection, String>>),
    UnsplashCollectionPhotos(Result<CollectionPhotos, String>),
    RequestUnsplash(isize),
    PauseUnsplash,
    OpenUrl(String),
}

impl BackgroundHandle {
    pub fn new(config: &Config, size: Size) -> (Self, Task<Message>) {
        let mut handle = Self {
            mode: config.background_mode,
            background: config.background.clone(),
            size,

            image_handle: None,

            unsplash_key: config.unsplash_key.clone(),
            unsplash_client: None,
            unsplash_state: None,
        };

        let task = handle.refresh(true);

        (handle, task)
    }

    pub fn load_config(&mut self, config: &Config, size: Size) -> Task<Message> {
        self.mode = config.background_mode;
        self.background = config.background.clone();
        self.size = size;

        if self.unsplash_key != config.unsplash_key {
            self.unsplash_key = config.unsplash_key.clone();
            self.unsplash_state = None;
            self.refresh(true)
        } else {
            self.refresh(false)
        }
    }

    fn refresh(&mut self, refresh_unsplash: bool) -> Task<Message> {
        debug!(
            "refreshing background (mode={}, background={})",
            self.mode, &self.background
        );

        match self.mode {
            #[cfg(not(target_arch = "wasm32"))]
            BackgroundMode::Local => {
                let path = self.background.clone();

                Task::future(async move { tokio::fs::read(&path).await })
                    .map(|r| Message::BackgroundRead(r.map_err(|e| e.to_string())))
            }
            BackgroundMode::Unsplash => {
                if !refresh_unsplash {
                    return Task::none();
                }

                if let Some(key) = &self.unsplash_key {
                    self.unsplash_client = match UnsplashClient::new(key) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            error!("failed to create Unsplash client: {e}");

                            return Task::none();
                        }
                    };

                    let collection = self.background.clone();
                    let client = self.unsplash_client.clone().unwrap();

                    Task::future(async move { client.collection(&collection).await }).map(|r| {
                        Message::UnsplashCollection(Box::new(r.map_err(|e| e.to_string())))
                    })
                } else {
                    Task::none()
                }
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
                }
                Ok(bytes) => {
                    self.image_handle = Some(image::Handle::from_bytes(bytes));
                    Task::none()
                }
            },
            Message::UnsplashCollection(res) => match *res {
                Err(e) => {
                    error!("failed to fetch collection: {e}");
                    Task::none()
                }
                Ok(collection) => {
                    self.unsplash_state = Some(UnsplashState {
                        collection: collection.id,
                        current: 0,
                        total: collection.total_photos,
                        paused: false,

                        current_page: 0,
                        current_page_photos: None,
                    });

                    Task::done(Message::RequestUnsplash(0))
                }
            },
            Message::RequestUnsplash(direction) => {
                match (&self.unsplash_client, &mut self.unsplash_state) {
                    (Some(client), Some(state)) => {
                        if state.paused {
                            return Task::none();
                        }

                        let mut new = state.current as isize + direction;

                        if new < 0 {
                            new = state.total as isize;
                        } else if new > state.total as isize {
                            new = 0;
                        }

                        state.current = new as usize;

                        let page = (state.current / 10) + 1;

                        if page == state.current_page && state.current_page_photos.is_some() {
                            return Task::done(Message::UnsplashCollectionPhotos(Ok(state
                                .current_page_photos
                                .as_ref()
                                .unwrap()
                                .clone())));
                        }

                        let collection = state.collection.clone();
                        let client = client.clone();

                        Task::future(async move {
                            client
                                .collection_photos(
                                    &collection,
                                    Some(CollectionPhotosOptions {
                                        page: Some(page),
                                        per_page: Some(10),
                                        ..Default::default()
                                    }),
                                )
                                .await
                        })
                        .map(|r| Message::UnsplashCollectionPhotos(r.map_err(|e| e.to_string())))
                    }
                    _ => Task::none(),
                }
            }
            Message::UnsplashCollectionPhotos(res) => match res {
                Err(e) => {
                    error!("failed to fetch collection photos: {e}");
                    Task::none()
                }
                Ok(photos) => match (&self.unsplash_client, &mut self.unsplash_state) {
                    (Some(client), Some(state)) => {
                        state.current_page_photos = Some(photos.clone());
                        state.current_page = (state.current / 10) + 1;

                        let idx = state.current % 10;
                        let photo = match photos.photos.get(idx) {
                            Some(photo) => photo,
                            None => {
                                error!("photo not found, current={}", state.current);
                                return Task::none();
                            }
                        };

                        let client = client.clone();
                        let photo = photo.clone();
                        let size = self.size;

                        Task::future(async move {
                            client
                                .download_photo(
                                    &photo,
                                    Some(PhotoFetchOptions {
                                        fm: Some(Format::Png),
                                        w: Some(size.width.round().into()),
                                        h: Some(size.height.round().into()),
                                        ..Default::default()
                                    }),
                                )
                                .await
                                .map(|b| b.to_vec())
                        })
                        .map(|r| Message::BackgroundRead(r.map_err(|e| e.to_string())))
                    }
                    _ => Task::none(),
                },
            },
            Message::PauseUnsplash => {
                if let Some(state) = &mut self.unsplash_state {
                    state.paused = !state.paused;
                    Task::none()
                } else {
                    Task::none()
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::OpenUrl(url) => {
                if let Err(e) = open::that_detached(url) {
                    error!("failed to open link: {e}")
                }

                Task::none()
            }
            #[cfg(target_arch = "wasm32")]
            Message::OpenUrl(url) => {
                if let Some(window) = web_sys::window() {
                    if window.open_with_url(&url).is_err() {
                        error!("failed to open link")
                    }
                }

                Task::none()
            }
        }
    }

    fn solid<'a>(color: Color) -> Element<'a, Message> {
        container("")
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::background(color))
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

                    #[cfg(not(target_arch = "wasm32"))]
                    if self.mode == BackgroundMode::Local {
                        return img.into();
                    }

                    if let Some(state) = &self.unsplash_state {
                        let idx = state.current % 10;
                        if let Some(photo) = state
                            .current_page_photos
                            .as_ref()
                            .and_then(|c| c.photos.get(idx))
                        {
                            let suffix = "?utm_source=fjordgard&utm_medium=referral";

                            let photo_url = format!("{}{suffix}", photo.links.html);

                            let user = &photo.user;

                            let author = format!(
                                "{}{}",
                                user.first_name,
                                user.last_name
                                    .as_ref()
                                    .map(|l| format!(" {l}"))
                                    .unwrap_or_default()
                            );
                            let author_url = format!("{}{suffix}", user.links.html);

                            stack![
                                img,
                                container(
                                    row![
                                        button(text("Photo").color(Color::WHITE))
                                            .style(button::text)
                                            .on_press_with(move || Message::OpenUrl(
                                                photo_url.clone()
                                            )),
                                        text(".").color(Color::WHITE),
                                        button(text(author).color(Color::WHITE))
                                            .style(button::text)
                                            .on_press_with(move || Message::OpenUrl(
                                                author_url.clone()
                                            )),
                                        text(".").color(Color::WHITE),
                                        button(text("Unsplash").color(Color::WHITE))
                                            .style(button::text)
                                            .on_press_with(move || Message::OpenUrl(format!(
                                                "https://unsplash.com/{suffix}"
                                            ))),
                                    ]
                                    .spacing(0)
                                )
                                .align_left(Length::Fill)
                                .align_bottom(Length::Fill)
                                .padding(15)
                            ]
                            .into()
                        } else {
                            img.into()
                        }
                    } else {
                        img.into()
                    }
                } else {
                    Self::solid(Color::BLACK)
                }
            }
        }
    }
}
