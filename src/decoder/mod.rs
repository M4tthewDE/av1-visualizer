use std::path::PathBuf;

use anyhow::Result;

mod mp4;

#[tracing::instrument]
pub fn decode(p: PathBuf) -> Result<()> {
    let _data = mp4::extract(p)?;

    Ok(())
}
