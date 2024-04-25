use std::io::{Cursor, Read};

use anyhow::Result;
use tracing::info;

use crate::decoder::mp4::{fixed16, fixed32};

#[derive(Clone, Debug, Default)]
pub struct Mvhd {
    pub version: u8,
    pub timescale: u32,
    pub duration: u32,
    pub rate: f64,
    pub volume: f64,
}

#[tracing::instrument(skip_all)]
pub fn parse_mvhd(c: &mut Cursor<Vec<u8>>, size: u64) -> Result<Mvhd> {
    let mut version = [0u8; 1];
    c.read_exact(&mut version)?;
    c.set_position(c.position() + 11);
    let mut timescale = [0u8; 4];
    c.read_exact(&mut timescale)?;
    let mut duration = [0u8; 4];
    c.read_exact(&mut duration)?;
    let mut rate = [0u8; 4];
    c.read_exact(&mut rate)?;
    let mut volume = [0u8; 2];
    c.read_exact(&mut volume)?;

    // FIXME: this skips 'trak'
    c.set_position(c.position() + size - 26);

    let mvhd = Mvhd {
        version: version[0],
        timescale: u32::from_be_bytes(timescale),
        duration: u32::from_be_bytes(duration),
        rate: fixed32(rate),
        volume: fixed16(volume),
    };
    info!("mvhd: {mvhd:?}");

    Ok(mvhd)
}
