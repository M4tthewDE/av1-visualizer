use std::path::Path;

use tracing::info;

mod mp4;

#[tracing::instrument]
pub fn decode(path: &Path) {
    info!("decoding");
    let _data = mp4::extract(path);
}
