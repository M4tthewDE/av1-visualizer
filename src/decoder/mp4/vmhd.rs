use std::io::{Cursor, Read};

use anyhow::Result;
use tracing::info;

#[derive(Clone, Debug, Default)]
pub struct Vmhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub graphics_mode: u16,
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl Vmhd {
    #[tracing::instrument(skip_all, name = "vmhd")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Vmhd> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut graphics_mode = [0u8; 2];
        c.read_exact(&mut graphics_mode)?;

        let mut red = [0u8; 2];
        c.read_exact(&mut red)?;

        let mut green = [0u8; 2];
        c.read_exact(&mut green)?;

        let mut blue = [0u8; 2];
        c.read_exact(&mut blue)?;

        let vmhd = Vmhd {
            version: version[0],
            flags,
            graphics_mode: u16::from_be_bytes(graphics_mode),
            red: u16::from_be_bytes(red),
            green: u16::from_be_bytes(green),
            blue: u16::from_be_bytes(blue),
        };

        info!("vmhd: {vmhd:?}");

        Ok(vmhd)
    }
}
