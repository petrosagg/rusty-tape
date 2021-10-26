use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kasetophono::{blogger, Subcategory, SubcategoryKind};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cassette {
    pub uuid: Uuid,
    pub name: String,
    pub safe_name: String,
    pub path: String,
    pub url: String,
    pub yt_url: String,
    pub videos: Vec<Video>,
    pub image_url: Option<String>,
    pub labels: Vec<String>,
    pub subcategories: Vec<Subcategory>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Video;

impl Cassette {
    pub fn try_from_entry(entry: blogger::Entry) -> Option<Self> {
        let iframe_selector = Selector::parse("iframe").unwrap();
        let image_selector = Selector::parse("img").unwrap();

        let content = Html::parse_fragment(&entry.content.t);
        let url = content
            .select(&iframe_selector)
            .next()
            .and_then(|e| e.value().attr("src"));

        if let Some(yt_url) = url.filter(|u| u.contains("youtube.com") && u.contains("list")) {
            let name = entry.title.t.trim();
            // Some cassettes contain slashes in their names
            let safe_name = name.replace('/', "-");
            let url = entry.link[2].href.clone();

            let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, url.as_bytes());

            // Extract year and date from URL like this: https://www.kasetophono.com/2019/01/nero.html
            let mut path: Vec<&str> = url
                .split('/')
                .rev()
                .skip(1)
                .take(2)
                .chain(Some("cassettes"))
                .collect();
            path.reverse();
            path.push(&safe_name);
            let path = path.join("/");

            let labels = entry
                .category
                .into_iter()
                .map(|c| c.term)
                .collect::<Vec<_>>();

            let image = content
                .select(&image_selector)
                .next()
                .and_then(|e| e.value().attr("src"));

            Some(Cassette {
                uuid,
                name: name.to_string(),
                safe_name,
                path,
                subcategories: vec![],
                labels,
                image_url: image.map(|s| s.to_string()),
                url,
                yt_url: yt_url.to_string(),
                videos: vec![],
                created_at: entry.published.t,
            })
        } else {
            None
        }
    }

    pub fn fill_subcategories(&mut self, subcategories: &[Subcategory]) {
        self.subcategories = subcategories
            .iter()
            .filter(|sc| match &sc.kind {
                SubcategoryKind::Label(l) => self.labels.contains(l),
                SubcategoryKind::Cassette(u) => self.url == *u,
            })
            .cloned()
            .collect();
    }
}
