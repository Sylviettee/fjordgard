#[derive(Debug, Clone, PartialEq, strum::Display, strum::VariantArray)]
pub enum BackgroundMode {
    Unsplash,
    Solid,
    Local,
}

impl BackgroundMode {
    pub fn default_background(&self) -> &'static str {
        match self {
            // https://unsplash.com/collections/1053828/tabliss-official
            Self::Unsplash => "1053828",
            Self::Solid => "#ffffff",
            Self::Local => "",
        }
    }

    pub fn edit_text(&self) -> &'static str {
        match self {
            Self::Unsplash => "Unsplash collection",
            Self::Solid => "Color (#rrggbb)",
            Self::Local => "File path",
        }
    }
}

#[derive(Clone)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct Config {
    pub time_format: String,
    pub background_mode: BackgroundMode,
    pub background: String,
    pub location: Option<Location>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_format: String::from("%-I:%M:%S"),
            background_mode: BackgroundMode::Unsplash,
            background: BackgroundMode::Unsplash.default_background().to_string(),
            location: None,
        }
    }
}
