use std::process::Command;
use std::os::unix::fs::symlink;
use std::fs::{self, File};
use std::io::prelude::*;
use std::iter::FromIterator;
use std::collections::{HashSet, HashMap};
use serde_json::Value;
use scraper::{Html, Selector};
use percent_encoding::percent_decode_str;

#[derive(Debug)]
struct Cassette {
    name: String,
    url: String,
    yt_url: String,
    labels: Vec<String>,
    created_at: String,
}

#[derive(Debug)]
struct Category {
    name: String,
    subcategories: Vec<Subcategory>,
}

#[derive(Debug)]
struct Subcategory {
    name: String,
    parent: String,
    kind: SubcategoryKind,
}

#[derive(Debug)]
enum SubcategoryKind {
    Label(String),
    Cassette(String),
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let body = reqwest::get("https://kasetophono.com").await?.text().await?;
    let selector = Selector::parse("ul#nav2 li a").unwrap();
    let content = Html::parse_document(&body);

    let mut subcategories = vec![];

    let mut categories = HashMap::new();
    for element in content.select(&selector) {
        let href = element.value().attr("href").unwrap();
        if href.contains("/p/") {
            let name = element.text().next().unwrap().trim().trim_start_matches('_');

            let name = name.chars()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 {
                        c.to_uppercase().next().unwrap()
                    } else {
                        c.to_lowercase().next().unwrap()
                    }
                })
                .collect::<String>();

            categories.entry(href).or_insert(name);
        }
    }

    for (url, category) in &mut categories {
        let body = reqwest::get(*url).await?.text().await?;
        let selector = Selector::parse("div.post-body h1.favourite-posts-title a").unwrap();
        let content = Html::parse_document(&body);

        for element in content.select(&selector) {
            let name = element.text().next().unwrap().trim().to_string();
            let href = element.value().attr("href").unwrap();

            let kind = if href.contains("/label/") {
                let label_raw = href.split_at(href.rfind('/').unwrap() + 1).1;
                SubcategoryKind::Label(percent_decode_str(label_raw).decode_utf8().unwrap().into_owned())
            } else {
                SubcategoryKind::Cassette(href.to_string())
            };

            let subcategory = Subcategory{
                name: name,
                parent: category.clone(),
                kind: kind,
            };

            subcategories.push(subcategory);
        }
    }

    // populate the downloaded cassettes from previous runs
    fs::create_dir_all("./cassettes").unwrap();
    let entries = fs::read_dir("./cassettes/")
        .unwrap()
        .map(|res| res.unwrap().file_name().into_string().unwrap());
    let downloaded_cassettes: HashSet<String> = HashSet::from_iter(entries);

    let mut index = 1;
    let mut cassettes = Vec::new();

    let selector = Selector::parse("iframe").unwrap();
    // Start gathering all cassettes until we hit the first one that is already downloaded
    'outer: loop {
        let url = format!("https://www.kasetophono.com/feeds/posts/default?alt=json&start-index={}", index);
        let body = reqwest::get(&url).await?.text().await?;

        let v: Value = serde_json::from_str(&body).unwrap();

        match v["feed"]["entry"].as_array() {
            Some(entries) => {
                index += entries.len();

                for entry in entries {
                    let content = Html::parse_fragment(entry["content"]["$t"].as_str().unwrap());
                    let url = content.select(&selector).next().and_then(|e| e.value().attr("src"));

                    if let Some(yt_url) = url.filter(|u| u.contains("videoseries")) {
                        let name = entry["title"]["$t"].as_str().unwrap().trim();
                        if downloaded_cassettes.contains(name) {
                            println!("found already downloaded cassette {}", name);
                            break 'outer;
                        }

                        let labels = entry["category"]
                            .as_array()
                            .map(|cats| {
                                cats.iter()
                                    .filter_map(|c| c["term"].as_str())
                                    .collect()
                            }).unwrap_or(vec![]);
                        
                        let published = entry["published"]["$t"].as_str().unwrap();

                        let url = entry["link"][2]["href"].as_str().unwrap().to_string();

                        let cassette = Cassette{
                            name: name.to_string(),
                            labels: labels.into_iter().map(String::from).collect(),
                            url: url,
                            yt_url: yt_url.to_string(),
                            created_at: published.to_string(),
                        };

                        println!("{:#?}", cassette);
                        cassettes.push(cassette);
                    }
                }
            }
            None => break
        }
    }

    // remove any partial downloads from previous runs
    fs::remove_dir_all(".tmp-cassette").unwrap_or(());

    for cassette in cassettes.iter().rev() {
        fs::create_dir(".tmp-cassette").unwrap();

        Command::new("youtube-dl")
            .current_dir(".tmp-cassette")
            .args(&[
                "--extract-audio",
                "--audio-format", "m4a",
                "--audio-quality", "128",
                "--add-metadata",
                "--embed-thumbnail",
                "--geo-bypass-country", "GR",
                "--ignore-errors",
                "--output", "%(playlist_index)s - %(title)s.%(ext)s",
                &cassette.yt_url,
            ])
            .status()
            .expect("failed to execute youtube-dl");

        let mut buf = Vec::new();
        writeln!(buf, "#EXTM3U").unwrap();
        writeln!(buf, "#PLAYLIST:{}", cassette.name).unwrap();
        writeln!(buf, "#EXT-X-PROGRAM-DATE-TIME:{}", cassette.created_at).unwrap();
        let mut songs = fs::read_dir(".tmp-cassette")
            .unwrap()
            .map(|res| res.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<String>>();
        songs.sort_unstable();
        for song in songs {
            writeln!(buf, "{}", song).unwrap();
        };

        let mut file = File::create(".tmp-cassette/playlist.m3u").unwrap();
        file.write_all(&buf).unwrap();

        let path = format!("cassettes/{}", cassette.name);
        fs::rename(".tmp-cassette", &path).unwrap();

        for label in &cassette.labels {
            // First write the playlsit in the tag directory
            let label_path = format!("labels/{}", label);
            fs::create_dir_all(&label_path).unwrap();
            symlink(format!("../../{}", path), format!("{}/{}", &label_path, cassette.name)).unwrap();

            // Then add cassette to subcategories that are of this label or include the cassette
            let subcategories = subcategories.iter().filter(|sc| {
                match &sc.kind {
                    SubcategoryKind::Label(l) => *label == *l,
                    SubcategoryKind::Cassette(u) => cassette.url == *u,
                }
            });

            for subcategory in subcategories {
                let subcategory_path = format!("categories/{}/{}", subcategory.parent, subcategory.name);
                fs::create_dir_all(&subcategory_path).unwrap();
                symlink(format!("../../../{}", path), format!("{}/{}", &subcategory_path, cassette.name)).unwrap();
            }
        }
    }

    Ok(())
}
