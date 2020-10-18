use std::path::PathBuf;
use std::str;
use std::str::FromStr;
use std::process::Command;
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

fn correct_loudness(path: &str, l: LoudNorm) {
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
            "-i", path,
            "-filter_complex", &filter,
            "-map_metadata", "0",
            "-map_metadata:s:a:0", "0:s:a:0",
            "-map_chapters", "0",
            "-map", "[norm0]",
            "-c:a", "aac",
            "-vn", "-sn",
            ".ffmpeg-workdir/audio.m4a"
        ])
        .status()
        .expect("Failed to run ffmpeg");

    if !status.success() {
        panic!();
    }

    fs::rename(".ffmpeg-workdir/audio.m4a", path).unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_audio() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test-audio.m4a");

        let loudnorm = measure_loudness(path.to_str().expect("Invalid path"));

        let expected = LoudNorm{
            input_i: -14.0,
            input_tp: -0.16,
            input_lra: 1.1,
            input_thresh: -24.03,
            target_offset: 0.35,
        };

        assert_eq!(loudnorm, expected);
    }

    #[test]
    fn correct_audio() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test-audio.m4a");

        let loudnorm = measure_loudness(path.to_str().expect("Invalid path"));

        correct_loudness(path.to_str().expect("Invalid path"), loudnorm);
    }
}
