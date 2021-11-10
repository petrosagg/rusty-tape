use percent_encoding::percent_decode_str;
use scraper::{Html, Selector};

use crate::{Subcategory, SubcategoryKind};

/// Extracts the list of categories from the frontpage of kasetophono.com
pub fn scrape_subcategories(document: &str) -> Result<Vec<Subcategory>, anyhow::Error> {
    let content = Html::parse_document(document);
    let selector = Selector::parse("div.post-body h1.favourite-posts-title a").unwrap();

    let mut subcategories = vec![];

    for element in content.select(&selector) {
        let name = element.text().next().unwrap().trim().to_string();
        let href = element.value().attr("href").unwrap();

        let kind = if href.contains("/label/") {
            let label_raw = href.split_at(href.rfind('/').unwrap() + 1).1;
            SubcategoryKind::Label(
                percent_decode_str(label_raw)
                    .decode_utf8()
                    .unwrap()
                    .into_owned(),
            )
        } else {
            SubcategoryKind::Cassette(href.to_string())
        };

        subcategories.push(Subcategory { name, kind });
    }

    Ok(subcategories)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correct_parse() {
        let body = include_str!("../../assets/category.html");
        let subcategories = scrape_subcategories(&body).unwrap();

        let label_subcategory = Subcategory {
            name: "Βαλκάνια".into(),
            kind: SubcategoryKind::Label("Balkan".into()),
        };
        assert!(subcategories.contains(&label_subcategory));

        let cassette_subcategory = Subcategory {
            name: "Ινδίες".into(),
            kind: SubcategoryKind::Cassette("https://www.kasetophono.com/2019/01/nero.html".into()),
        };
        assert!(subcategories.contains(&cassette_subcategory));

        assert_eq!(subcategories.len(), 18);
    }
}
