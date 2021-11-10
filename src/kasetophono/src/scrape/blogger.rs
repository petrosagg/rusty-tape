use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document<'a> {
    pub version: &'a str,
    pub encoding: &'a str,
    #[serde(borrow)]
    pub feed: Feed<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed<'a> {
    pub xmlns: &'a str,
    #[serde(rename = "xmlns$openSearch")]
    pub xmlns_open_search: &'a str,
    #[serde(rename = "xmlns$blogger")]
    pub xmlns_blogger: &'a str,
    #[serde(rename = "xmlns$georss")]
    pub xmlns_georss: &'a str,
    #[serde(rename = "xmlns$gd")]
    pub xmlns_gd: &'a str,
    #[serde(rename = "xmlns$thr")]
    pub xmlns_thr: &'a str,
    pub id: Id<'a>,
    #[serde(borrow)]
    pub updated: Updated<'a>,
    #[serde(borrow)]
    pub category: Vec<Category<'a>>,
    #[serde(borrow)]
    pub title: Title<'a>,
    #[serde(borrow)]
    pub subtitle: Subtitle<'a>,
    #[serde(borrow)]
    pub link: Vec<Link<'a>>,
    #[serde(borrow)]
    pub author: Vec<Author<'a>>,
    #[serde(borrow)]
    pub generator: Generator<'a>,
    #[serde(borrow, rename = "openSearch$totalResults")]
    pub open_search_total_results: OpenSearchTotalResults<'a>,
    #[serde(borrow, rename = "openSearch$startIndex")]
    pub open_search_start_index: OpenSearchStartIndex<'a>,
    #[serde(rename = "openSearch$itemsPerPage")]
    pub open_search_items_per_page: OpenSearchItemsPerPage<'a>,
    #[serde(borrow, default)]
    pub entry: Vec<Entry<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Updated<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title<'a> {
    #[serde(rename = "type")]
    pub type_field: &'a str,
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtitle<'a> {
    #[serde(rename = "type")]
    pub type_field: &'a str,
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link<'a> {
    pub rel: &'a str,
    #[serde(rename = "type")]
    pub type_field: Option<&'a str>,
    pub href: &'a str,
    pub title: Option<&'a str>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author<'a> {
    #[serde(borrow)]
    pub name: Name<'a>,
    #[serde(borrow)]
    pub uri: Option<Uri<'a>>,
    #[serde(borrow)]
    pub email: Email<'a>,
    #[serde(borrow, rename = "gd$image")]
    pub gd_image: GdImage<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Email<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GdImage<'a> {
    pub rel: &'a str,
    pub width: &'a str,
    pub height: &'a str,
    pub src: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generator<'a> {
    pub version: &'a str,
    pub uri: &'a str,
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSearchTotalResults<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSearchStartIndex<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSearchItemsPerPage<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry<'a> {
    #[serde(borrow)]
    pub id: Id<'a>,
    #[serde(borrow)]
    pub published: Published<'a>,
    #[serde(borrow)]
    pub updated: Updated<'a>,
    #[serde(borrow, default)]
    pub category: Vec<Category<'a>>,
    #[serde(borrow)]
    pub title: Title<'a>,
    #[serde(borrow)]
    pub content: Content<'a>,
    #[serde(borrow)]
    pub link: Vec<Link<'a>>,
    #[serde(borrow)]
    pub author: Vec<Author<'a>>,
    #[serde(borrow, rename = "media$thumbnail")]
    pub media_thumbnail: Option<MediaThumbnail<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Published<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category<'a> {
    #[serde(default)]
    pub scheme: Option<&'a str>,
    pub term: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content<'a> {
    #[serde(rename = "type")]
    pub type_field: &'a str,
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uri<'a> {
    #[serde(rename = "$t")]
    pub t: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaThumbnail<'a> {
    #[serde(rename = "xmlns$media")]
    pub xmlns_media: &'a str,
    pub url: &'a str,
    pub height: &'a str,
    pub width: &'a str,
}
