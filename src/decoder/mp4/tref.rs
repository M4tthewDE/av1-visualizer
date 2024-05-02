use std::io::{Cursor, Read};

use anyhow::Result;

#[derive(Clone, Debug, Default)]
pub struct Tref {
    pub version: u8,
    pub flags: [u8; 3],
    pub reference_type: String,
    pub track_ids: Vec<u32>,
}

impl Tref {
    #[tracing::instrument(skip_all, name = "tref")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Tref> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut reference_type = [0u8; 4];
        c.read_exact(&mut reference_type)?;
        let reference_type = String::from_utf8(reference_type.to_vec())?;

        let mut track_ids = Vec::new();
        for _ in 0..(size - 8) / 8 {
            let mut track_id = [0u8; 4];
            c.read_exact(&mut track_id)?;
            track_ids.push(u32::from_be_bytes(track_id));
        }

        Ok(Tref {
            version: version[0],
            flags,
            reference_type,
            track_ids,
        })
    }
}
