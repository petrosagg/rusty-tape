use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{Headers, Json};
use http::{header::HeaderName, Uri};
use include_dir::{include_dir, Dir};
use parking_lot::RwLock;
use uuid::Uuid;

use kasetophono::Cassette;

use crate::ServerState;

static ROOT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

include!(concat!(env!("OUT_DIR"), "/paths.rs"));

pub async fn play(Path(uuid): Path<Uuid>, Extension(state): Extension<Arc<RwLock<ServerState>>>) {
    let mut state = state.write();

    if let Some(mut handle) = state.mpv_process.take() {
        handle.kill().expect("failed to kill previous mpv process");
    }
    let cassette = &state.cassettes[&uuid];
    println!("playing {}", &cassette.name);
    let handle = std::process::Command::new(MPV)
        .args(&["--no-video", "--shuffle", &cassette.yt_url])
        .spawn()
        .unwrap();
    state.mpv_process = Some(handle);
}

pub async fn stop(Extension(state): Extension<Arc<RwLock<ServerState>>>) {
    println!("stopping");
    let mut state = state.write();
    if let Some(mut handle) = state.mpv_process.take() {
        handle.kill().expect("failed to kill mpv process");
    }
}

pub async fn list(
    Extension(state): Extension<Arc<RwLock<ServerState>>>,
) -> Json<HashMap<Uuid, Cassette>> {
    let state = state.read();
    Json(state.cassettes.clone())
}

pub async fn fallback(
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
