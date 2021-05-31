use std::path::Path;

use image::ImageFormat;

pub fn is_supported(path: &Path) -> bool {
    ImageFormat::from_path(path).is_ok()
}
