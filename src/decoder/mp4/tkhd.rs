use std::io::{Cursor, Read};

use anyhow::Result;
use tracing::info;

use crate::decoder::mp4::{fixed16, fixed32};

#[derive(Clone, Debug, Default)]
pub struct Tkhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub creation_time: u32,
    pub modification_time: u32,
    pub id: u32,
    pub duration: u32,
    pub layer: u16,
    pub alternate_group: u16,
    pub volume: f64,
    pub matrix: [u32; 9],
    pub width: f64,
    pub height: f64,
}

impl Tkhd {
    #[tracing::instrument(skip_all, name = "tkhd")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Tkhd> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut creation_time = [0u8; 4];
        c.read_exact(&mut creation_time)?;

        let mut modification_time = [0u8; 4];
        c.read_exact(&mut modification_time)?;

        let mut id = [0u8; 4];
        c.read_exact(&mut id)?;

        c.set_position(c.position() + 4);

        let mut duration = [0u8; 4];
        c.read_exact(&mut duration)?;

        let mut layer = [0u8; 2];
        c.read_exact(&mut layer)?;

        let mut alternate_group = [0u8; 2];
        c.read_exact(&mut alternate_group)?;

        let mut volume = [0u8; 2];
        c.read_exact(&mut volume)?;

        c.set_position(c.position() + 10);

        let mut matrix = [0_u32; 9];
        for m in &mut matrix {
            let mut val = [0u8; 4];
            c.read_exact(&mut val)?;
            *m = u32::from_be_bytes(val);
        }

        let mut width = [0u8; 4];
        c.read_exact(&mut width)?;

        let mut height = [0u8; 4];
        c.read_exact(&mut height)?;

        let tkhd = Tkhd {
            version: version[0],
            flags,
            creation_time: u32::from_be_bytes(creation_time),
            modification_time: u32::from_be_bytes(modification_time),
            id: u32::from_be_bytes(id),
            duration: u32::from_be_bytes(duration),
            layer: u16::from_be_bytes(layer),
            alternate_group: u16::from_be_bytes(alternate_group),
            volume: fixed16(volume),
            matrix,
            width: fixed32(width),
            height: fixed32(height),
        };
        info!("tkhd: {tkhd:?}");

        Ok(tkhd)
    }
}
