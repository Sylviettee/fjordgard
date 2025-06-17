use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum UnsplashResponse {
    Success(serde_json::Value),
    Error { errors: Vec<String> },
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
