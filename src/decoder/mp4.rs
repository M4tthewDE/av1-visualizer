use std::path::Path;

use tracing::info;

// https://github.com/alfg/mp4/blob/master/atom/box.go#L99
#[tracing::instrument(skip_all)]
pub fn extract(_p: &Path) {
    info!("extracting");
}
