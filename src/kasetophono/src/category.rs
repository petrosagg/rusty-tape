use scraper::{Html, Selector};
use std::collections::HashSet;

/// A top level category of kasetophono
#[derive(Debug, PartialEq)]
pub struct Category {
    /// The name of the category
    pub name: String,
    /// The URL of the category blogpost containing the subcategories
    pub url: String,
}

/// Extracts the list of categories from the frontpage of kasetophono.com
pub fn scrape_categories(document: &str) -> Result<Vec<Category>, anyhow::Error> {
    // We're looking for this kind of elements:
    // <ul id='nav2'>
    // <li><a href='https://www.kasetophono.com/p/blog-page_28.html'>Ξενα</a></li>
    // <li><a href='http://www.kasetophono.com/search/label/Playlist'>_Νέες</a></li>
    let content = Html::parse_document(document);
    let category_selector = Selector::parse("ul#nav2 li a").unwrap();

    let mut categories = vec![];
    let mut seen_urls = HashSet::new();
    for element in content.select(&category_selector) {
        // The category menu structure of kasetophono.com is defined by whether or not the link
        // text starts with an underscore or not. The underscore signifies that the item will be
        // listed in the dropdown of its parent, and its parent is the first element before that
        // does not have an underscore.
        //
        // The menu structure is generally consistent with the URL structure. Categories are
        // generally blogposts that their url is `/p/<page>` and subcategories are generally label
        // search and their url is `/search/<label>`.
        //
        // An exception to the rule above are some subcategories that are actually blogposts. When
        // we encounter a subcategory that has a category-type URL we check if we have seen a
        // normal category with the same URL before. If we have, we ignore it. Otherwise, we will
        // process it as if it was a top level category.
        //
        // The anomalous subcategories at the time of writing are "Κι άλλα μουσικά είδη" and "4
        // Εποχές"
        let url = element.value().attr("href").unwrap().to_owned();
        if url.contains("/p/") && seen_urls.insert(url.clone()) {
            // It's unclear why but some names begin with an underscore, so trim it
            let raw_name = element
                .text()
                .next()
                .unwrap()
                .trim()
                .trim_start_matches('_');

            // Category names have inconsistent capitalization, so we normalize them here
            let mut name = String::with_capacity(raw_name.len());
            let mut chars = raw_name.chars();
            name.extend(chars.next().into_iter().flat_map(char::to_uppercase));
            name.extend(chars.flat_map(char::to_lowercase));
            match name.pop() {
                Some('σ') => name.push('ς'),
                Some(c) => name.push(c),
                None => {}
            }

            categories.push(Category { name, url });
        }
    }

    Ok(categories)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correct_parse() {
        let body = include_str!("../../assets/frontpage.html");
        let categories = scrape_categories(&body).unwrap();

        // check we chose the correct name for the double category
        let double_sub = Category {
            name: "Ξενα".into(),
            url: "https://www.kasetophono.com/p/blog-page_28.html".into(),
        };
        assert!(categories.contains(&double_sub));

        // check that 'σ' -> 'ς' is working
        let name_check = Category {
            name: "Μερες".into(),
            url: "https://www.kasetophono.com/p/blog-page_3.html".into(),
        };
        assert!(categories.contains(&name_check));

        // check that we hoisted anomalous subcategory
        let fake_category = Category {
            name: "4 εποχές".into(),
            url: "https://www.kasetophono.com/p/4.html".into(),
        };
        assert!(categories.contains(&fake_category));

        // check that we got the correct number of categories
        assert_eq!(categories.len(), 10);
    }
}
