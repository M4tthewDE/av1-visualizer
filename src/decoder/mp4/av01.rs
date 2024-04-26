use anyhow::{bail, Result};
use std::io::{Cursor, Read};
use tracing::info;

use super::av1c::Av1C;

#[derive(Clone, Debug, Default)]
pub struct Av01 {
    pub width: u16,
    pub height: u16,
    pub horizresolution: u32,
    pub vertresolution: u32,
    pub frame_count: u16,
    pub compressor_name: String,
    pub depth: u16,
    pub av1c: Av1C,
}

impl Av01 {
    #[tracing::instrument(skip_all, name = "av01")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Av01> {
        // skip pre_defined and reserved
        c.set_position(c.position() + 16);

        let mut width = [0u8; 2];
        c.read_exact(&mut width)?;
        let width = u16::from_be_bytes(width);

        let mut height = [0u8; 2];
        c.read_exact(&mut height)?;
        let height = u16::from_be_bytes(height);

        let mut horizresolution = [0u8; 4];
        c.read_exact(&mut horizresolution)?;
        let horizresolution = u32::from_be_bytes(horizresolution);

        let mut vertresolution = [0u8; 4];
        c.read_exact(&mut vertresolution)?;
        let vertresolution = u32::from_be_bytes(vertresolution);

        // skip reserved
        c.set_position(c.position() + 4);

        let mut frame_count = [0u8; 2];
        c.read_exact(&mut frame_count)?;
        let frame_count = u16::from_be_bytes(frame_count);

        let mut compressor_name = [0u8; 4];
        c.read_exact(&mut compressor_name)?;
        let compressor_name = String::from_utf8(compressor_name.to_vec())?;

        // unsure why we need to skip 28 bytes here
        c.set_position(c.position() + 28);

        let mut depth = [0u8; 2];
        c.read_exact(&mut depth)?;
        let depth = u16::from_be_bytes(depth);

        // skip pre_defined
        c.set_position(c.position() + 2);

        let mut size = [0u8; 4];
        c.read_exact(&mut size)?;
        let size = u32::from_be_bytes(size);

        let mut config_box = [0u8; 4];
        c.read_exact(&mut config_box)?;
        let config_box = String::from_utf8(config_box.to_vec())?;

        if config_box != "av1C" {
            bail!("config box {config_box} is not supported");
        }

        let av1c = Av1C::new(c, size as u64)?;

        let av01 = Av01 {
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            compressor_name,
            depth,
            av1c,
        };

        info!("av01: {av01:?}");

        Ok(av01)
    }
}
