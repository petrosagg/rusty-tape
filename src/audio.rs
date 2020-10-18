use std::path::PathBuf;
use std::str;
use std::str::FromStr;
use std::process::{Command, Stdio};
use serde::Deserialize;
use std::fs;
use serde::Deserializer;
use serde::de;
use serde_json::Value;

#[derive(Debug,Copy,Clone,PartialEq,Deserialize)]
struct LoudNorm {
    #[serde(deserialize_with = "de_fromstr")]
    input_i: f32,
    #[serde(deserialize_with = "de_fromstr")]
    input_tp: f32,
    #[serde(deserialize_with = "de_fromstr")]
    input_lra: f32,
    #[serde(deserialize_with = "de_fromstr")]
    input_thresh: f32,
    #[serde(deserialize_with = "de_fromstr")]
    target_offset: f32,
}

fn de_fromstr<'de, D: Deserializer<'de>, T: FromStr>(deserializer: D) -> Result<T, D::Error>  {
    let value = Value::deserialize(deserializer)?;
    let s = value.as_str().ok_or(de::Error::custom("invalid type"))?;
    s.parse().or(Err(de::Error::custom("invalid value")))
}

fn measure_loudness(path: &str) -> LoudNorm {
    let output = Command::new("ffmpeg")
        .args(&[
            "-nostdin", "-nostats", "-y",
            "-i", path,
            "-filter_complex", "[0:0]loudnorm=i=-23.0:lra=7.0:tp=-2.0:offset=0.0:print_format=json",
            "-vn", "-sn",
            "-f", "null",
            "/dev/null",
        ])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run ffmpeg");

    let mut stderr = str::from_utf8(&output.stderr).expect("Invalid utf8 in ffmpeg output").lines();

    while let Some(line) = stderr.next() {
        if line.contains("Parsed_loudnorm") {
            break;
        }
    }

    let result = stderr.collect::<String>();
    
    serde_json::from_str(&result).expect("Invalid ffmpeg json")
}

fn correct_loudness(input: &str, output: &str, l: LoudNorm) {
    // values taken from ffmpeg-normalize with default arguments
    let filter = format!("[0:0]loudnorm=i=-23.0:\
                          lra=7.0:\
                          tp=-2.0:\
                          offset={}:\
                          measured_i={}:\
                          measured_lra={}:\
                          measured_tp={}:\
                          measured_thresh={}:\
                          linear=true:\
                          print_format=json[norm0]", l.target_offset, l.input_i, l.input_lra, l.input_tp, l.input_thresh);

    // remove any partial downloads from previous runs
    fs::remove_dir_all(".ffmpeg-workdir").unwrap_or(());
    fs::create_dir(".ffmpeg-workdir").unwrap();

    let status = Command::new("ffmpeg")
        .args(&[
            "-nostdin", "-nostats", "-y",
            "-i", input,
            "-filter_complex", &filter,
            "-map_metadata", "0",
            "-map_metadata:s:a:0", "0:s:a:0",
            "-map_chapters", "0",
            "-map", "[norm0]",
            "-c:a", "libmp3lame",
            "-q:a", "2",
            "-vn", "-sn",
            ".ffmpeg-workdir/audio.mp3"
        ])
        .stderr(Stdio::null())
        .status()
        .expect("Failed to run ffmpeg");

    if !status.success() {
        panic!();
    }

    fs::rename(".ffmpeg-workdir/audio.mp3", output).unwrap();
}

fn add_cassette_metadata(input: &str, output: &str, album_name: &str, track_n: u8, track_total: u8, album_art_path: &str) {
    let album_metadata = format!("album={}", album_name);
    let track_metadata = format!("track={}/{}", track_n, track_total);

    let status = Command::new("ffmpeg")
        .args(&[
            "-nostdin", "-nostats", "-y",
            "-i", input,
            "-i", album_art_path,
            "-map", "0:0",
            "-map", "1:0",
            "-c", "copy",
            "-c:v", "png",
            "-id3v2_version", "3",
            "-metadata:s:v", "title=Album cover",
            "-metadata:s:v", "comment=Cover (front)",
            "-metadata", &album_metadata,
            "-metadata", &track_metadata,
            ".ffmpeg-workdir/audio.mp3"
        ])
        .stderr(Stdio::null())
        .status()
        .expect("Failed to run ffmpeg");

    if !status.success() {
        panic!();
    }

    fs::rename(".ffmpeg-workdir/audio.mp3", output).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_audio() {
        let mut input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input.push("resources/test-audio.mp3");
        let input = input.to_str().expect("Invalid path");

        let loudnorm = measure_loudness(input);

        let expected = LoudNorm{
            input_i: -14.01,
            input_tp: -0.21,
            input_lra: 1.1,
            input_thresh: -24.03,
            target_offset: 0.35
        };

        assert_eq!(loudnorm, expected);
    }

    #[test]
    fn correct_audio() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let mut input = base.clone();
        input.push("resources/test-audio.mp3");
        let input = input.to_str().expect("Invalid path");

        let mut output = base.clone();
        output.push("resources/test-correct_audio.mp3");
        let output = output.to_str().expect("Invalid path");

        let loudnorm = measure_loudness(input);

        correct_loudness(input, output, loudnorm);
    }

    #[test]
    fn correct_metadata() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let mut input = base.clone();
        input.push("resources/test-audio.mp3");
        let input = input.to_str().expect("Invalid path");

        let mut output = base.clone();
        output.push("resources/test-correct_metadata.mp3");
        let output = output.to_str().expect("Invalid path");

        let mut img_path = base.clone();
        img_path.push("resources/album-art.gif");
        let img_path = img_path.to_str().expect("Invalid path");

        add_cassette_metadata(input, output, "My album", 3, 10, img_path);
    }
}
