use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloggerDocument {
    pub version: String,
    pub encoding: String,
    pub feed: Feed,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub xmlns: String,
    #[serde(rename = "xmlns$openSearch")]
    pub xmlns_open_search: String,
    #[serde(rename = "xmlns$blogger")]
    pub xmlns_blogger: String,
    #[serde(rename = "xmlns$georss")]
    pub xmlns_georss: String,
    #[serde(rename = "xmlns$gd")]
    pub xmlns_gd: String,
    #[serde(rename = "xmlns$thr")]
    pub xmlns_thr: String,
    pub id: Id,
    pub updated: Updated,
    pub category: Vec<Category>,
    pub title: Title,
    pub subtitle: Subtitle,
    pub link: Vec<Link>,
    pub author: Vec<Author>,
    pub generator: Generator,
    #[serde(rename = "openSearch$totalResults")]
    pub open_search_total_results: OpenSearchTotalResults,
    #[serde(rename = "openSearch$startIndex")]
    pub open_search_start_index: OpenSearchStartIndex,
    #[serde(rename = "openSearch$itemsPerPage")]
    pub open_search_items_per_page: OpenSearchItemsPerPage,
    pub entry: Vec<Entry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Updated {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtitle {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub href: String,
    pub title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: Name,
    pub uri: Option<Uri>,
    pub email: Email,
    #[serde(rename = "gd$image")]
    pub gd_image: GdImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GdImage {
    pub rel: String,
    pub width: String,
    pub height: String,
    pub src: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    pub version: String,
    pub uri: String,
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSearchTotalResults {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSearchStartIndex {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSearchItemsPerPage {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: Id,
    pub published: Published,
    pub updated: Updated,
    #[serde(default)]
    pub category: Vec<Category>,
    pub title: Title,
    pub content: Content,
    pub link: Vec<Link>,
    pub author: Vec<Author>,
    #[serde(rename = "media$thumbnail")]
    pub media_thumbnail: Option<MediaThumbnail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Published {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[serde(default)]
    pub scheme: Option<String>,
    pub term: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uri {
    #[serde(rename = "$t")]
    pub t: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaThumbnail {
    #[serde(rename = "xmlns$media")]
    pub xmlns_media: String,
    pub url: String,
    pub height: String,
    pub width: String,
}
