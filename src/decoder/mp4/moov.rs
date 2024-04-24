use anyhow::Result;
use std::io::{Cursor, Read};
use tracing::info;

use anyhow::bail;

#[derive(Clone, Debug, Default)]
pub struct Moov {
    pub mvhd: Mvhd,
}

#[tracing::instrument(skip_all)]
pub fn moov(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Moov> {
    let mut mvhd: Mvhd;
    loop {
        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;
        let box_size = u32::from_be_bytes(box_size);

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;

        match box_type.as_str() {
            "mvhd" => mvhd = parse_mvhd(c, box_size as u64)?,
            typ => bail!("box type {typ:?} not implemented"),
        }

        if c.position() == size as u64 {
            break;
        }
    }

    Ok(Moov { mvhd })
}

#[derive(Clone, Debug, Default)]
pub struct Mvhd {
    pub version: u8,
    pub timescale: u32,
    pub duration: u32,
    pub rate: f64,
    pub volume: f64,
}

#[tracing::instrument(skip_all)]
fn parse_mvhd(c: &mut Cursor<Vec<u8>>, size: u64) -> Result<Mvhd> {
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

fn fixed32(data: [u8; 4]) -> f64 {
    u32::from_be_bytes(data) as f64 / 65536.0
}

fn fixed16(data: [u8; 2]) -> f64 {
    u16::from_be_bytes(data) as f64 / 256.0
}
