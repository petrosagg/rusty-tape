use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;

use futures::stream::{self, StreamExt, TryStreamExt};
use futures::TryFutureExt;
use log::{debug, info};
use uuid::Uuid;
use warp::Filter;

mod kasetophono;

use kasetophono::{blogger, Cassette, Category, Subcategory};

type Result<T> = std::result::Result<T, anyhow::Error>;

async fn subcategories(categories: &[Category]) -> Result<Vec<Subcategory>> {
    let mut responses = stream::iter(categories)
        .map(|c| reqwest::get(&c.url).and_then(|r| r.text()))
        .buffer_unordered(5);

    let mut subcategories = vec![];
    while let Some(response) = responses.try_next().await? {
        let subs = kasetophono::scrape_subcategories(&response)?;
        subcategories.extend(subs);
    }
    Ok(subcategories)
}

async fn cassettes(subcategories: &[Subcategory]) -> Result<HashMap<Uuid, Cassette>> {
    const PAGE_SIZE: usize = 25;
    let mut responses = stream::iter((1..).step_by(PAGE_SIZE))
        .map(|page| {
            reqwest::get(format!(
                "https://www.kasetophono.com/feeds/posts/default?alt=json\
                &start-index={}\
                &max-results={}",
                page, PAGE_SIZE,
            ))
            .and_then(|r| r.text())
        })
        .buffer_unordered(5);

    let mut cassettes = HashMap::new();
    while let Some(response) = responses.try_next().await? {
        let document: blogger::Document = serde_json::from_str(&response).unwrap();

        let mut empty = true;
        for entry in document.feed.entry {
            if let Some(mut cassette) = Cassette::try_from_entry(entry) {
                empty = false;
                cassette.fill_subcategories(subcategories);
                cassettes.insert(cassette.uuid, cassette);
            }
        }
        if empty {
            break;
        }
    }

    let total = cassettes.len();
    debug!("fetched {} cassettes", total);

    let mut i = 0;
    cassettes.retain(move |_uuid, cassette| {
        i += 1;
        debug!("fetching songs ({}/{}): {:?}", i, total, &cassette.name);
        match cassette.fill_songs() {
            Ok(()) => true,
            Err(e) => {
                debug!("discarding cassette: {:?}: {}",&cassette.name, e);
                false
            }
        }
    });

    Ok(cassettes)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cassettes: HashMap<Uuid, Cassette> =
        if let Ok(content) = File::open("metadata.json") {
            info!("loading cassettes from disk");
            serde_json::from_reader(content).unwrap()
        } else {
            info!("loading cassettes from upstream");
            let body = reqwest::get("https://www.kasetophono.com")
                .await?
                .text()
                .await?;

            let categories = kasetophono::scrape_categories(&body).unwrap();
            let subcategories = subcategories(&categories).await?;
            let cassettes = cassettes(&subcategories).await?;

            let buf = serde_json::to_string_pretty(&cassettes).unwrap();
            let mut file = File::create("metadata.json").unwrap();
            file.write_all(buf.as_bytes()).unwrap();
            cassettes
        };

    info!("setting up http server");
    let cassettes = Arc::new(cassettes);
    let state: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    let play_state = Arc::clone(&state);
    let play = warp::path!("api" / "play" / Uuid).map(move |uuid: Uuid| {
        let cassette = &cassettes[&uuid];

        println!("playing {}", &cassette.name);
        let mut state = play_state.lock().unwrap();
        if let Some(mut handle) = state.take() {
            handle.kill().unwrap();
        }
        let handle = std::process::Command::new("mpv")
            .args(&["--no-video", &cassette.yt_url])
            .spawn()
            .unwrap();
        *state = Some(handle);
        format!("{:?}", &cassette.name)
    });

    let stop_state = Arc::clone(&state);
    let stop = warp::path!("api" / "stop").map(move || {
        println!("stopping");
        let mut state = stop_state.lock().unwrap();
        if let Some(mut handle) = state.take() {
            handle.kill().unwrap();
        }
        "Killed"
    });

    let cassettes = warp::path!("api" / "cassettes")
        .and(warp::fs::file("metadata.json"))
        .with(warp::compression::gzip());

    let routes = warp::get().and(
        play.or(stop)
            .or(cassettes)
            .or(warp::fs::dir("frontend/dist")),
    );

    info!("ready to accept connections");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
