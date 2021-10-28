use serde::{Deserialize, Serialize};
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub duration: Option<u64>,
}

impl Song {
    pub fn audio_url(&self) -> Option<String> {
        match YoutubeDl::new(&self.id).run().unwrap() {
            YoutubeDlOutput::SingleVideo(video) => video
                .formats
                .into_iter()
                .flatten()
                .filter(|f| f.acodec.as_deref() == Some("opus")).next()?.url,
            _ => return None,
        }
    }
}
