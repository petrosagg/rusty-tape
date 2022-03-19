use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{Headers, Json};
use axum::routing::get;
use axum::Router;
use futures::stream::{self, StreamExt, TryStreamExt};
use futures::TryFutureExt;
use http::{header::HeaderName, Uri};
use include_dir::{include_dir, Dir};
use log::{debug, info};
use uuid::Uuid;

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

async fn load_cassettes() -> Result<HashMap<Uuid, Cassette>> {
    let body = reqwest::get("https://www.kasetophono.com")
        .await?
        .text()
        .await?;

    let categories = kasetophono::scrape::category::scrape_categories(&body)?;

    let subcategories = subcategories(&categories).await?;
    let cassettes = cassettes(&subcategories).await?;
    Ok(cassettes)
}

async fn play_handler(Path(uuid): Path<Uuid>, Extension(state): Extension<Arc<ServerState>>) {
    let cassette = &state.cassettes[&uuid];

    println!("playing {}", &cassette.name);
    let mut mpv_process = state.mpv_process.lock().expect("lock poisoned");
    if let Some(mut handle) = mpv_process.take() {
        handle.kill().expect("failed to kill previous mpv process");
    }
    let handle = std::process::Command::new(MPV)
        .args(&["--no-video", "--shuffle", &cassette.yt_url])
        .spawn()
        .unwrap();
    *mpv_process = Some(handle);
}

async fn stop_handler(Extension(state): Extension<Arc<ServerState>>) {
    println!("stopping");
    let mut mpv_process = state.mpv_process.lock().expect("lock poisoned");
    if let Some(mut handle) = mpv_process.take() {
        handle.kill().expect("failed to kill mpv process");
    }
}

async fn list_handler(
    Extension(state): Extension<Arc<ServerState>>,
) -> Json<HashMap<Uuid, Cassette>> {
    Json(state.cassettes.clone())
}

async fn static_handler(
    uri: Uri,
) -> std::result::Result<(Headers<[(HeaderName, String); 1]>, &'static [u8]), StatusCode> {
    let path = if uri.path() == "/" {
        "index.html"
    } else {
        &uri.path()[1..]
    };
    let content_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .essence_str()
        .to_owned();
    let body = match ROOT.get_file(&path) {
        Some(file) => file.contents(),
        None => return Err(StatusCode::NOT_FOUND),
    };
    let headers = Headers([(http::header::CONTENT_TYPE, content_type)]);
    Ok((headers, body))
}

struct ServerState {
    cassettes: HashMap<Uuid, Cassette>,
    mpv_process: Mutex<Option<Child>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cassettes = loop {
        info!("loading cassettes from upstream");
        match load_cassettes().await {
            Ok(res) => break res,
            Err(err) => {
                info!("failed to get cassettes from upstream: {}", err);
                tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
            }
        }
    };

    info!("setting up http server");
    let server_state = Arc::new(ServerState {
        cassettes,
        mpv_process: Mutex::new(None),
    });

    let app = Router::new()
        .route("/api/play/:uuid", get(play_handler))
        .route("/api/stop", get(stop_handler))
        .route("/api/cassettes", get(list_handler))
        .layer(Extension(server_state))
        .fallback(get(static_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to bind socket");

    Ok(())
}
