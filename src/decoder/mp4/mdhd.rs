use anyhow::bail;
use std::io::{Cursor, Read};
use tracing::info;

use anyhow::Result;

#[derive(Clone, Debug, Default)]
pub struct Mdhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,
    pub language: String,
}

impl Mdhd {
    #[tracing::instrument(skip_all, name = "mdhd")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Mdhd> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut creation_time = [0u8; 4];
        c.read_exact(&mut creation_time)?;

        let mut modification_time = [0u8; 4];
        c.read_exact(&mut modification_time)?;

        let mut timescale = [0u8; 4];
        c.read_exact(&mut timescale)?;

        let mut duration = [0u8; 4];
        c.read_exact(&mut duration)?;

        let mut language = [0u8; 2];
        c.read_exact(&mut language)?;
        let language = u16::from_be_bytes(language);

        let mut lang = String::new();
        for i in (0..3).rev() {
            lang.push((((language >> (i * 5)) & 31) as u8 | 0x60) as char);
        }

        let mut pre_defined = [0u8; 2];
        c.read_exact(&mut pre_defined)?;
        let pre_defined = u16::from_be_bytes(pre_defined);

        if pre_defined != 0 {
            bail!("pre_defined has to be 0, is {pre_defined}");
        }

        let mdhd = Mdhd {
            version: version[0],
            flags,
            creation_time: u32::from_be_bytes(creation_time),
            modification_time: u32::from_be_bytes(modification_time),
            timescale: u32::from_be_bytes(timescale),
            duration: u32::from_be_bytes(duration),
            language: lang,
        };
        info!("mdhd: {mdhd:?}");

        Ok(mdhd)
    }
}
