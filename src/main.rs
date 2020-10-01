use std::process::Command;
use std::os::unix::fs::symlink;
use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;
use scraper::{Html, Selector};
use percent_encoding::percent_decode_str;

#[derive(Debug,Serialize)]
struct Cassette<'a> {
    name: String,
    safe_name: String,
    path: String,
    url: String,
    yt_url: String,
    labels: Vec<String>,
    subcategories: Vec<&'a Subcategory>,
    created_at: String,
}

#[derive(Debug,Serialize)]
struct Subcategory {
    name: String,
    category: String,
    kind: SubcategoryKind,
}

#[derive(Debug,Serialize)]
enum SubcategoryKind {
    Label(String),
    Cassette(String),
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let body = reqwest::get("https://kasetophono.com").await?.text().await?;
    let selector = Selector::parse("ul#nav2 li a").unwrap();
    let content = Html::parse_document(&body);

    let mut all_subcategories = vec![];

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
                category: category.clone(),
                kind: kind,
            };

            all_subcategories.push(subcategory);
        }
    }

    let mut index = 1;
    let mut cassettes = Vec::new();

    let selector = Selector::parse("iframe").unwrap();
    loop {
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
                        // Some cassettes contain slashes in their names
                        let safe_name = name.replace('/', "-");
                        let url = entry["link"][2]["href"].as_str().unwrap().to_string();

                        // Extract year and date from URL like this: https://www.kasetophono.com/2019/01/nero.html
                        let mut path: Vec<&str> = url.split('/').rev().skip(1).take(2).chain(Some("cassettes")).collect();
                        path.reverse();
                        path.push(&safe_name);
                        let path = path.join("/");

                        let published = entry["published"]["$t"].as_str().unwrap();

                        let labels = entry["category"]
                            .as_array()
                            .map(|cats| {
                                cats.iter()
                                    .filter_map(|c| c["term"].as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            }).unwrap_or(vec![]);

                        let subcategories = all_subcategories.iter().filter(|sc| {
                            match &sc.kind {
                                SubcategoryKind::Label(l) => labels.contains(l),
                                SubcategoryKind::Cassette(u) => url == *u,
                            }
                        }).collect();

                        let cassette = Cassette{
                            name: name.to_string(),
                            safe_name: safe_name,
                            path: path,
                            subcategories: subcategories,
                            labels: labels,
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

    {
        let buf = serde_json::to_string(&cassettes).unwrap();
        let mut file = File::create("metadata.json").unwrap();
        file.write_all(buf.as_bytes()).unwrap();
    }

    // remove any partial downloads from previous runs
    fs::remove_dir_all(".tmp-cassette").unwrap_or(());

    for cassette in cassettes.iter().rev() {
        if Path::new(&cassette.path).exists() {
            println!("Skipping already downloaded cassette {}", cassette.name);
            continue;
        }
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

        fs::create_dir_all(Path::new(&cassette.path).parent().unwrap()).unwrap();
        fs::rename(".tmp-cassette", &cassette.path).unwrap();

        for label in &cassette.labels {
            let label_path = format!("labels/{}", label);
            fs::create_dir_all(&label_path).unwrap();
            let src = format!("../../{}", cassette.path);
            let dest = format!("{}/{}", &label_path, cassette.safe_name);
            println!("Symlinking {} -> {}", dest, src);
            symlink(src, dest).unwrap();
        }

        for subcategory in &cassette.subcategories {
            let subcategory_path = format!("categories/{}/{}", subcategory.category, subcategory.name);
            fs::create_dir_all(&subcategory_path).unwrap();
            let src = format!("../../../{}", cassette.path);
            let dest = format!("{}/{}", &subcategory_path, cassette.safe_name);
            println!("Symlinking {} -> {}", dest, src);
            symlink(src, dest).unwrap();
        }
    }

    Ok(())
}
