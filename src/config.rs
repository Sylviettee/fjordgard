pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
    pub name: Option<String>
}

pub struct Config {
    pub timezone: String,
    pub time_format: String,
    pub collection: String,
    pub location: Option<Location>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timezone: String::from("Etc/UTC"),
            time_format: String::from("%-I:%M:%S"),
            // https://unsplash.com/collections/1053828/tabliss-official
            collection: String::from("1053828"),
            location: None
        }
    }
}
