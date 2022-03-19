use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Child;
use std::sync::Arc;

use axum::extract::Extension;
use axum::routing::get;
use axum::Router;
use futures::stream::{self, StreamExt, TryStreamExt};
use futures::TryFutureExt;
use log::{debug, info};
use parking_lot::RwLock;
use uuid::Uuid;

use kasetophono::{scrape::blogger, Cassette, Category, Subcategory};

mod handlers;

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

pub struct ServerState {
    cassettes: HashMap<Uuid, Cassette>,
    mpv_process: Option<Child>,
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
    let server_state = Arc::new(RwLock::new(ServerState {
        cassettes,
        mpv_process: None,
    }));

    let app = Router::new()
        .route("/api/play/:uuid", get(handlers::play))
        .route("/api/stop", get(handlers::stop))
        .route("/api/cassettes", get(handlers::list))
        .layer(Extension(server_state))
        .fallback(get(handlers::fallback));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to bind socket");

    Ok(())
}
