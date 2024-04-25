use std::io::{Cursor, Read};

use anyhow::{bail, Result};
use tracing::info;

#[derive(Clone, Debug, Default)]
pub struct Elst {
    pub version: u8,
    pub flags: [u8; 3],
    pub entry_count: u32,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub segment_duration: u32,
    pub media_time: i32,
    pub media_rate: i16,
}

impl Elst {
    #[tracing::instrument(skip_all)]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Elst> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut entry_count = [0u8; 4];
        c.read_exact(&mut entry_count)?;
        let entry_count = u32::from_be_bytes(entry_count);

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let mut segment_duration = [0u8; 4];
            c.read_exact(&mut segment_duration)?;
            let segment_duration = u32::from_be_bytes(segment_duration);

            let mut media_time = [0u8; 4];
            c.read_exact(&mut media_time)?;
            let media_time = i32::from_be_bytes(media_time);

            let mut media_rate_integer = [0u8; 2];
            c.read_exact(&mut media_rate_integer)?;
            let media_rate_integer = i16::from_be_bytes(media_rate_integer);

            let mut media_rate_fraction = [0u8; 2];
            c.read_exact(&mut media_rate_fraction)?;
            let media_rate_fraction = i16::from_be_bytes(media_rate_fraction);

            if media_rate_fraction != 0 {
                bail!("invalid media_rate_fraction: {media_rate_fraction}");
            }

            entries.push(Entry {
                segment_duration,
                media_time,
                media_rate: media_rate_integer,
            })
        }

        let elst = Elst {
            version: version[0],
            flags,
            entry_count,
            entries,
        };

        info!("elst: {elst:?}");
        bail!("TODO: elst");
    }
}
