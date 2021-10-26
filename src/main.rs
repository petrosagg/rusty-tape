use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;

use futures::stream::{StreamExt, TryStreamExt};
use futures::TryFutureExt;
use uuid::Uuid;
use warp::Filter;

mod kasetophono;

use kasetophono::{blogger, Cassette, Category, Subcategory};

async fn subcategories(categories: &[Category]) -> Result<Vec<Subcategory>, anyhow::Error> {
    let mut responses = futures::stream::iter(categories)
        .map(|c| reqwest::get(&c.url).and_then(|r| r.text()))
        .buffer_unordered(5);

    let mut subcategories = vec![];
    while let Some(response) = responses.try_next().await? {
        let subs = kasetophono::scrape_subcategories(&response)?;
        subcategories.extend(subs);
    }
    Ok(subcategories)
}

async fn cassettes(
    subcategories: &[Subcategory],
) -> Result<HashMap<Uuid, Cassette>, anyhow::Error> {
    const PAGE_SIZE: usize = 25;
    let mut responses = futures::stream::iter((0..).step_by(PAGE_SIZE))
        .map(|page| async move {
            let url = format!(
                "https://www.kasetophono.com/feeds/posts/default?alt=json&start-index={}&max-results={}",
                page + 1,
                PAGE_SIZE,
            );
            reqwest::get(&url).and_then(|r| r.text()).await
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
                cassettes.insert(cassette.uuid.clone(), cassette);
            }
        }
        if empty {
            break;
        }
    }
    Ok(cassettes)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let cassettes: HashMap<Uuid, Cassette> =
        if let Ok(content) = std::fs::read_to_string("metadata.json") {
            println!("Loading cassettes from disk");
            serde_json::from_str(&content).unwrap()
        } else {
            println!("Loading cassettes from upstream");
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
        format!("Killed")
    });

    let cassettes = warp::path!("api" / "cassettes")
        .and(warp::fs::file("metadata.json"))
        .with(warp::compression::gzip());

    let routes = warp::get().and(
        play.or(stop)
            .or(cassettes)
            .or(warp::fs::dir("frontend/dist")),
    );

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    // // remove any partial downloads from previous runs
    // fs::remove_dir_all(".tmp-cassette").unwrap_or(());

    // for cassette in cassettes.iter().rev() {
    //     if Path::new(&cassette.path).exists() {
    //         println!("Skipping already downloaded cassette {}", cassette.name);
    //         continue;
    //     }
    //     fs::create_dir(".tmp-cassette").unwrap();

    //     let status = Command::new("youtube-dl")
    //         .current_dir(".tmp-cassette")
    //         .args(&[
    //             "--extract-audio",
    //             "--audio-format", "mp3",
    //             "--audio-quality", "2",
    //             "--add-metadata",
    //             "--geo-bypass-country", "GR",
    //             "--ignore-errors",
    //             "--output", "%(playlist_index)s - %(title)s.%(ext)s",
    //             &cassette.yt_url,
    //         ])
    //         .status()
    //         .expect("failed to execute youtube-dl");

    //     if !status.success() {
    //         panic!();
    //     }

    //     if let Some(url) = &cassette.image_url {
    //         let data = reqwest::get(url).await?.bytes().await?;
    //         let mut thumb = File::create(".tmp-cassette/thumbnail.gif").unwrap();

    //         thumb.write_all(data.as_ref()).unwrap();
    //     }

    //     let mut songs = fs::read_dir(".tmp-cassette")
    //         .unwrap()
    //         .map(|res| res.unwrap().path().to_str().unwrap().to_string())
    //         .filter(|p| !p.contains("thumbnail.gif"))
    //         .collect::<Vec<_>>();
    //     songs.sort_unstable();

    //     let total = songs.len() as u8;
    //     songs.par_iter().enumerate().for_each(|(i, song)| {
    //         println!("Normalizing track: {}", song);
    //         let l = audio::measure_loudness(song);
    //         audio::correct_loudness(song, song, l);
    //         audio::add_cassette_metadata(song, song, &cassette, (i + 1) as u8, total, ".tmp-cassette/thumbnail.gif");
    //     });

    //     fs::create_dir_all(Path::new(&cassette.path).parent().unwrap()).unwrap();
    //     fs::rename(".tmp-cassette", &cassette.path).unwrap();

    //     for label in &cassette.labels {
    //         let label_path = format!("labels/{}", label);
    //         fs::create_dir_all(&label_path).unwrap();
    //         let src = format!("../../{}", cassette.path);
    //         let dest = format!("{}/{}", &label_path, cassette.safe_name);
    //         println!("Symlinking {} -> {}", dest, src);
    //         symlink(src, dest).unwrap();
    //     }

    //     for subcategory in &cassette.subcategories {
    //         let subcategory_path = format!("categories/{}/{}", subcategory.category, subcategory.name);
    //         fs::create_dir_all(&subcategory_path).unwrap();
    //         let src = format!("../../../{}", cassette.path);
    //         let dest = format!("{}/{}", &subcategory_path, cassette.safe_name);
    //         println!("Symlinking {} -> {}", dest, src);
    //         symlink(src, dest).unwrap();
    //     }
    // }

    Ok(())
}
