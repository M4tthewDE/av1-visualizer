use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

use self::mp4::Mp4;

mod mp4;

#[tracing::instrument]
pub fn decode(p: PathBuf) -> Result<()> {
    let mp4 = Mp4::new(p)?;
    info!("mp4: {mp4:?}");

    Ok(())
}
