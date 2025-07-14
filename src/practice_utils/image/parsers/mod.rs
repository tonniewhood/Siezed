pub mod bmp;
pub mod ppm;

use super::{Image, Pixel};
use std::path::Path;

pub fn load_from_path<P: AsRef<Path>>(filepath: P, no_aspect: bool) -> anyhow::Result<Image> {
    let path = filepath.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "ppm" => ppm::parse_ppm(path, no_aspect),
        "bmp" => bmp::parse_bmp(path, no_aspect),
        other => anyhow::bail!("Unsupported extension: '{}'", other),
    }
}
