use std::io::{Cursor, Read};

use anyhow::Result;

#[derive(Clone, Debug, Default)]
pub struct Gmhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub data: Vec<u8>,
}

impl Gmhd {
    #[tracing::instrument(skip_all, name = "gmhd")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: u32) -> Result<Gmhd> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        // 12:
        // box_size + box_type
        // version + flags
        let mut data = vec![0u8; size as usize - 12];
        c.read_exact(&mut data)?;

        Ok(Gmhd {
            version: version[0],
            flags,
            data,
        })
    }
}
