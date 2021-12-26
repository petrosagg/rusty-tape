use std::env;
use std::fs;
use std::path::Path;
use which::CanonicalPath;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let paths = Path::new(&out_dir).join("paths.rs");

    let mpv = CanonicalPath::new("mpv").expect("mpv missing from PATH");
    fs::write(&paths, format!("static MPV: &'static str = {:?};", mpv)).unwrap();

    println!("cargo:rerun-if-env-changed=PATH");
}
