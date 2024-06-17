use std::path::PathBuf;

use anyhow::{bail, Result};
use av1::Decoder;
use tracing::info;

use crate::decoder::ivf::Ivf;

use self::mp4::Mp4;

mod av1;
mod ivf;
mod mp4;

#[tracing::instrument]
pub fn decode(p: PathBuf) -> Result<()> {
    match p.extension() {
        Some(ext) => match ext.to_str() {
            Some("mp4") => decode_mp4(p),
            Some("ivf") => decode_ivf(p),
            _ => bail!("file extension {:?} is not supported", ext),
        },
        None => bail!(
            "input file {:?} has no extension, unable to determine decoder",
            p
        ),
    }
}

#[tracing::instrument(skip_all)]
pub fn decode_mp4(p: PathBuf) -> Result<()> {
    let mp4 = Mp4::new(p)?;
    info!("ftyp: {:?}", mp4.ftyp);
    info!("moov: {:?}", mp4.moov);
    if let Some(mdat) = &mp4.mdat {
        info!("mdat: {:?} bytes", mdat.len());
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn decode_ivf(p: PathBuf) -> Result<()> {
    let ivf = Ivf::new(p)?;
    info!("fourcc: {}", ivf.fourcc);
    info!("width: {}", ivf.width);
    info!("height: {}", ivf.height);
    info!("block 1: {}", ivf.blocks[0]);

    match ivf.fourcc.as_str() {
        "AV01" => {
            let mut decoder = Decoder::default();
            decoder.decode(ivf)
        }
        _ => panic!("unknown ivf fourcc: {}", ivf.fourcc),
    }
}
