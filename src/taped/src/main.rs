use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Child;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Extension;
use axum::routing::get;
use axum::Router;
use futures::stream::{self, StreamExt, TryStreamExt};
use futures::TryFutureExt;
use log::{debug, info};
use parking_lot::RwLock;
use tower_http::compression::CompressionLayer;
use uuid::Uuid;

use kasetophono::{scrape::blogger, Cassette, Category, Subcategory};

mod handlers;

type Result<T> = std::result::Result<T, anyhow::Error>;

async fn subcategories(categories: &[Category]) -> Result<Vec<Subcategory>> {
    let mut responses = stream::iter(categories)
        .map(|c| reqwest::get(&c.url).and_then(|r| r.text()))
        .buffer_unordered(5)
        // Workaround for rust-lang/rust#89976
        .boxed();

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
    while let Some(mut response) = responses.try_next().await? {
        let document: blogger::Document = match serde_json::from_str(&response) {
            Ok(document) => document,
            Err(_) => {
                // The website suddenly started serving responses with invalid JSON where two
                // objects are separated by two commas instead of one. Probably a bug somewhere in
                // Google. Workaround by retrying the deserialization after replacing all double
                // commas with single ones
                response = response.replace(",,", ",");
                serde_json::from_str(&response).unwrap()
            }
        };

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

async fn refresh_loop(state: Arc<RwLock<ServerState>>) {
    loop {
        info!("loading cassettes from upstream");
        match load_cassettes().await {
            Ok(cassettes) => {
                {
                    state.write().cassettes = cassettes;
                }
                // Refresh once a day
                tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
            }
            Err(err) => {
                info!("failed to get cassettes from upstream: {}", err);
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }
    }
}

#[derive(Default)]
pub struct ServerState {
    cassettes: HashMap<Uuid, Cassette>,
    mpv_process: Option<Child>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("setting up http server");
    let server_state = Arc::new(RwLock::new(ServerState::default()));

    tokio::spawn(refresh_loop(server_state.clone()));

    let app = Router::new()
        .route("/api/play/:uuid", get(handlers::play))
        .route("/api/stop", get(handlers::stop))
        .route("/api/cassettes", get(handlers::list))
        .layer(CompressionLayer::new())
        .layer(Extension(server_state))
        .fallback(get(handlers::fallback));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to bind socket");

    Ok(())
}
