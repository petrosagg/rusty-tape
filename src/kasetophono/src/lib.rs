use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "scrape")]
pub mod scrape;

/// A top level category of kasetophono
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    /// The name of the category
    pub name: String,
    /// The URL of the category blogpost containing the subcategories
    pub url: String,
}

/// A subcategory of kasetophono
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subcategory {
    /// The name of the subcategory
    pub name: String,
    /// The kind of subcategory
    pub kind: SubcategoryKind,
}

/// The kind of subcategory of kasetophono
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SubcategoryKind {
    /// A subcategory that is a search for a specific label
    Label(String),
    /// A subcategory that is just a single cassette
    Cassette(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cassette {
    pub uuid: Uuid,
    pub name: String,
    pub safe_name: String,
    pub path: String,
    pub url: String,
    pub yt_url: String,
    pub videos: Vec<Song>,
    pub image_url: Option<String>,
    pub labels: Vec<String>,
    pub subcategories: Vec<Subcategory>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub duration: Option<u64>,
}
