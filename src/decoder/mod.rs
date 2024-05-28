use std::path::PathBuf;

use anyhow::{bail, Result};
use tracing::info;

use self::mp4::Mp4;

mod mp4;

#[tracing::instrument]
pub fn decode(p: PathBuf) -> Result<()> {
    match p.extension() {
        Some(ext) => match ext.to_str() {
            Some("mp4") => decode_mp4(p),
            _ => bail!("file extension {:?} is not supported", ext),
        },
        None => bail!(
            "input file {:?} has no extension, unable to determine decoder",
            p
        ),
    }
}

#[tracing::instrument]
pub fn decode_mp4(p: PathBuf) -> Result<()> {
    let mp4 = Mp4::new(p)?;
    info!("ftyp: {:?}", mp4.ftyp);
    info!("moov: {:?}", mp4.moov);
    if let Some(mdat) = &mp4.mdat {
        info!("mdat: {:?} bytes", mdat.len());
    }

    Ok(())
}
