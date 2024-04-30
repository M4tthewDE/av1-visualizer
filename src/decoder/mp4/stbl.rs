use anyhow::{bail, Context};
use std::io::{Cursor, Read};
use tracing::info;

use anyhow::Result;

use super::stsd::Stsd;

pub type SampleEntry = (u32, u32);

#[derive(Clone, Debug, Default)]
pub struct Stts {
    pub version: u8,
    pub flags: [u8; 3],
    pub entries: Vec<SampleEntry>,
}

impl Stts {
    #[tracing::instrument(skip_all, name = "stbl")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Stts> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut entry_count = [0u8; 4];
        c.read_exact(&mut entry_count)?;
        let entry_count = u32::from_be_bytes(entry_count);

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let mut sample_count = [0u8; 4];
            c.read_exact(&mut sample_count)?;
            let sample_count = u32::from_be_bytes(sample_count);

            let mut sample_delta = [0u8; 4];
            c.read_exact(&mut sample_delta)?;
            let sample_delta = u32::from_be_bytes(sample_delta);

            entries.push((sample_count, sample_delta));
        }

        let stts = Stts {
            version: version[0],
            flags,
            entries,
        };

        info!("stts: {stts:?}");

        Ok(stts)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stbl {
    pub stsd: Stsd,
    pub stts: Stts,
}

impl Stbl {
    #[tracing::instrument(skip_all, name = "stbl")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Stbl> {
        let mut stsd = None;
        let mut stts = None;
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let _box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "stsd" => stsd = Some(Stsd::new(c)?),
                "stts" => stts = Some(Stts::new(c)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        let stbl = Stbl {
            stsd: stsd.context("no stsd found")?,
            stts: stts.context("no stts found")?,
        };

        info!("stbl: {stbl:?}");

        Ok(stbl)
    }
}
