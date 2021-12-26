use std::collections::HashMap;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;

use futures::stream::{self, StreamExt, TryStreamExt};
use futures::TryFutureExt;
use include_dir::{include_dir, Dir};
use log::{debug, info};
use uuid::Uuid;
use warp::{reply::Response, Filter};

use kasetophono::{scrape::blogger, Cassette, Category, Subcategory};

include!(concat!(env!("OUT_DIR"), "/paths.rs"));

static ROOT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

type Result<T> = std::result::Result<T, anyhow::Error>;

async fn subcategories(categories: &[Category]) -> Result<Vec<Subcategory>> {
    let mut responses = stream::iter(categories)
        .map(|c| reqwest::get(&c.url).and_then(|r| r.text()))
        .buffer_unordered(5);

    let mut subcategories = vec![];
    while let Some(response) = responses.try_next().await? {
        let subs = kasetophono::scrape::subcategory::scrape_subcategories(&response)?;
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

    // let mut i = 0;
    // cassettes.retain(move |_uuid, cassette| {
    //     i += 1;
    //     debug!("fetching songs ({}/{}): {:?}", i, total, &cassette.name);
    //     match cassette.fill_songs() {
    //         Ok(()) => true,
    //         Err(e) => {
    //             debug!("discarding cassette: {:?}: {}", &cassette.name, e);
    //             false
    //         }
    //     }
    // });

    Ok(cassettes)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("loading cassettes from upstream");
    let body = reqwest::get("https://www.kasetophono.com")
        .await?
        .text()
        .await?;

    let categories = kasetophono::scrape::category::scrape_categories(&body).unwrap();
    let subcategories = subcategories(&categories).await?;
    let cassettes = cassettes(&subcategories).await?;

    info!("setting up http server");
    let cassettes = Arc::new(cassettes);
    let state: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    let play_state = Arc::clone(&state);
    let play_cassettes = Arc::clone(&cassettes);
    let play = warp::path!("api" / "play" / Uuid).map(move |uuid: Uuid| {
        let cassette = &play_cassettes[&uuid];

        println!("playing {}", &cassette.name);
        let mut state = play_state.lock().unwrap();
        if let Some(mut handle) = state.take() {
            handle.kill().unwrap();
        }
        let handle = std::process::Command::new(MPV)
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

    let list_cassettes = Arc::clone(&cassettes);
    let list = warp::path!("api" / "cassettes")
        // TODO: This computes the json every time
        .map(move || warp::reply::json(&*list_cassettes))
        .with(warp::compression::gzip());

    let routes = warp::get().and(
        play.or(stop)
            .or(list)
            .or(warp::path::full()
                .and_then(|path: warp::path::FullPath| {
                    let path = path.as_str()[1..].to_owned();
                    let mime = mime_guess::from_path(&path).first_or_octet_stream();
                    async move {
                        let body = match ROOT.get_file(&path) {
                            Some(file) => file.contents(),
                            None => return Err(warp::reject::not_found()),
                        };
                        let mut resp = Response::new(body.into());
                        resp.headers_mut().insert(http::header::CONTENT_TYPE, mime.as_ref().try_into().unwrap());
                        Ok(resp)
                    }
                })
            )
    );

    info!("ready to accept connections");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
