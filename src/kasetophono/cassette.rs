use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kasetophono::{BloggerDocument, Subcategory, SubcategoryKind};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cassette {
    pub uuid: Uuid,
    pub name: String,
    pub safe_name: String,
    pub path: String,
    pub url: String,
    pub yt_url: String,
    pub image_url: Option<String>,
    pub labels: Vec<String>,
    pub subcategories: Vec<Subcategory>,
    pub created_at: String,
}

pub fn scrape_cassettes(
    doc: BloggerDocument,
    subcategories: &[Subcategory],
) -> Result<Vec<(Uuid, Cassette)>, anyhow::Error> {
    let mut cassettes = vec![];

    let iframe_selector = Selector::parse("iframe").unwrap();
    let image_selector = Selector::parse("img").unwrap();

    for entry in doc.feed.entry {
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

            let published = entry.published.t;

            let labels = entry
                .category
                .into_iter()
                .map(|c| c.term)
                .collect::<Vec<_>>();

            let subcategories = subcategories
                .iter()
                .cloned()
                .filter(|sc| match &sc.kind {
                    SubcategoryKind::Label(l) => labels.contains(l),
                    SubcategoryKind::Cassette(u) => url == *u,
                })
                .collect();

            let image = content
                .select(&image_selector)
                .next()
                .and_then(|e| e.value().attr("src"));

            let cassette = Cassette {
                uuid,
                name: name.to_string(),
                safe_name: safe_name,
                path: path,
                subcategories: subcategories,
                labels: labels,
                image_url: image.map(|s| s.to_string()),
                url: url,
                yt_url: yt_url.to_string(),
                created_at: published.to_string(),
            };

            cassettes.push((uuid, cassette));
        }
    }
    Ok(cassettes)
}
