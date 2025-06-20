use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize, Serializer};
use strum::Display;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum UnsplashResponse {
    Error { errors: Vec<String> },
    Success(serde_json::Value),
}

#[derive(Serialize)]
#[serde(rename = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
    Squarish,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
pub struct CollectionPhotosOptions {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub orientation: Option<Orientation>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CollectionPhotos {
    pub collection_total: usize,
    pub per_page: usize,
    pub photos: Vec<Photo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Photo {
    pub id: String,
    pub slug: String,
    pub alternative_slugs: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
    pub promoted_at: Option<String>,
    pub width: usize,
    pub height: usize,
    pub color: String,
    pub blur_hash: String,
    pub description: Option<String>,
    pub alt_description: Option<String>,
    pub urls: PhotoUrls,
    pub links: PhotoLinks,
    pub likes: usize,
    pub liked_by_user: bool,
    pub topic_submissions: HashMap<String, TopicSubmission>,
    pub asset_type: String,
    pub user: User,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PhotoUrls {
    pub raw: String,
    pub full: String,
    pub regular: String,
    pub small: String,
    pub thumb: String,
    pub small_s3: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PhotoLinks {
    #[serde(rename = "self")]
    pub this: String,
    pub html: String,
    pub download: String,
    pub download_location: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TopicSubmission {
    pub status: String,
    pub approved_on: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub updated_at: String,
    pub username: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub twitter_username: Option<String>,
    pub portfolio_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub links: UserLinks,
    pub profile_image: ProfileImageLinks,
    pub instagram_username: Option<String>,
    pub total_collections: usize,
    pub total_likes: usize,
    pub total_photos: usize,
    pub total_promoted_photos: usize,
    pub total_illustrations: usize,
    pub total_promoted_illustrations: usize,
    pub accepted_tos: bool,
    pub for_hire: bool,
    // a bit redundant aye unsplash
    pub social: UserSocials,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserLinks {
    #[serde(rename = "self")]
    pub this: String,
    pub html: String,
    pub photos: String,
    pub likes: String,
    pub portfolio: String,
    pub following: Option<String>,
    pub followers: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProfileImageLinks {
    pub small: String,
    pub medium: String,
    pub large: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserSocials {
    pub instagram_username: Option<String>,
    pub portfolio_url: Option<String>,
    pub twitter_username: Option<String>,
    pub paypal_email: Option<String>,
}

#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum Crop {
    Top,
    Bottom,
    Left,
    Right,
    Faces,
    FocalPoint,
    Edges,
    Entropy,
}

#[derive(Serialize)]
#[serde(rename = "lowercase")]
pub enum Format {
    Avif,
    Gif,
    Jp2,
    Jpg,
    Json,
    Jxr,
    PJpg,
    Mp4,
    Png,
    Png8,
    Png32,
    Webm,
    Webp,
    BlurHash,
}

#[derive(Serialize)]
#[serde(rename = "lowercase")]
pub enum Auto {
    Compress,
    Enhance,
    True,
    Format,
    Redeye,
}

#[derive(Serialize)]
#[serde(rename = "lowercase")]
pub enum Fit {
    Clamp,
    Clip,
    Crop,
    FaceArea,
    Fill,
    FillMax,
    Max,
    Min,
    Scale,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
pub struct PhotoFetchOptions {
    pub w: Option<f64>,
    pub h: Option<f64>,
    #[serde(serialize_with = "csv")]
    pub crop: Option<Vec<Crop>>,
    pub fm: Option<Format>,
    pub auto: Option<Auto>,
    pub q: Option<usize>,
    pub fit: Option<Fit>,
    pub dpr: Option<usize>,
}

fn csv<S: Serializer, T: Display>(list: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error> {
    if let Some(list) = list {
        let s: String = list
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");

        serializer.serialize_str(&s)
    } else {
        serializer.serialize_none()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CollectionLinks {
    #[serde(rename = "self")]
    pub this: String,
    pub html: String,
    pub photos: String,
    pub related: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PreviewPhoto {
    pub id: String,
    pub slug: String,
    pub created_at: String,
    pub updated_at: String,
    pub blur_hash: String,
    pub asset_type: String,
    pub urls: PhotoUrls,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub published_at: String,
    pub last_collected_at: String,
    pub updated_at: String,
    pub featured: bool,
    pub total_photos: usize,
    pub private: bool,
    pub share_key: String,
    pub links: CollectionLinks,
    pub user: User,
    pub cover_photo: Option<Photo>,
    pub preview_photos: Vec<PreviewPhoto>,
}
