use std::io::{Cursor, Read};

use anyhow::Result;
use tracing::info;

use crate::decoder::mp4::{fixed16, fixed32};

#[derive(Clone, Debug, Default)]
pub struct Mvhd {
    pub version: u8,
    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,
    pub rate: f64,
    pub volume: f64,
    pub matrix: [u32; 9],
    pub pre_defined: [u32; 6],
    pub next_track_id: u32,
}

#[tracing::instrument(skip_all)]
pub fn parse_mvhd(c: &mut Cursor<Vec<u8>>) -> Result<Mvhd> {
    let mut version = [0u8; 4];
    c.read_exact(&mut version)?;

    let mut creation_time = [0u8; 4];
    c.read_exact(&mut creation_time)?;

    let mut modification_time = [0u8; 4];
    c.read_exact(&mut modification_time)?;

    let mut timescale = [0u8; 4];
    c.read_exact(&mut timescale)?;

    let mut duration = [0u8; 4];
    c.read_exact(&mut duration)?;

    let mut rate = [0u8; 4];
    c.read_exact(&mut rate)?;

    let mut volume = [0u8; 2];
    c.read_exact(&mut volume)?;

    // skip 10 reserved bytes
    c.set_position(c.position() + 10);

    let mut matrix = [0_u32; 9];
    for m in &mut matrix {
        let mut val = [0u8; 4];
        c.read_exact(&mut val)?;
        *m = u32::from_be_bytes(val);
    }

    let mut pre_defined = [0_u32; 6];
    for p in &mut pre_defined {
        let mut val = [0u8; 4];
        c.read_exact(&mut val)?;
        *p = u32::from_be_bytes(val);
    }

    let mut next_track_id = [0u8; 4];
    c.read_exact(&mut next_track_id)?;

    let mvhd = Mvhd {
        version: version[0],
        creation_time: u32::from_be_bytes(creation_time),
        modification_time: u32::from_be_bytes(modification_time),
        timescale: u32::from_be_bytes(timescale),
        duration: u32::from_be_bytes(duration),
        rate: fixed32(rate),
        volume: fixed16(volume),
        matrix,
        pre_defined,
        next_track_id: u32::from_be_bytes(next_track_id),
    };
    info!("mvhd: {mvhd:?}");

    Ok(mvhd)
}
