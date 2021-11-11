use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use crate::Song;

impl Song {
    pub fn audio_url(&self) -> Option<String> {
        match YoutubeDl::new(&self.id).run().unwrap() {
            YoutubeDlOutput::SingleVideo(video) => {
                video
                    .formats
                    .into_iter()
                    .flatten()
                    .find(|f| f.acodec.as_deref() == Some("opus"))?
                    .url
            }
            _ => None,
        }
    }
}
