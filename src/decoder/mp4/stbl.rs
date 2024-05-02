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
    #[tracing::instrument(skip_all, name = "stts")]
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
pub struct Stss {
    pub version: u8,
    pub flags: [u8; 3],
    pub sample_numbers: Vec<u32>,
}

impl Stss {
    #[tracing::instrument(skip_all, name = "stss")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Stss> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut entry_count = [0u8; 4];
        c.read_exact(&mut entry_count)?;
        let entry_count = u32::from_be_bytes(entry_count);

        let mut sample_numbers = Vec::new();
        for _ in 0..entry_count {
            let mut sample_number = [0u8; 4];
            c.read_exact(&mut sample_number)?;
            sample_numbers.push(u32::from_be_bytes(sample_number));
        }

        let stss = Stss {
            version: version[0],
            flags,
            sample_numbers,
        };

        info!("stss: {stss:?}");

        Ok(stss)
    }
}

pub type StscEntry = (u32, u32, u32);

#[derive(Clone, Debug, Default)]
pub struct Stsc {
    pub version: u8,
    pub flags: [u8; 3],
    pub entries: Vec<StscEntry>,
}

impl Stsc {
    #[tracing::instrument(skip_all, name = "stsc")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Stsc> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut entry_count = [0u8; 4];
        c.read_exact(&mut entry_count)?;
        let entry_count = u32::from_be_bytes(entry_count);

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let mut first_chunk = [0u8; 4];
            c.read_exact(&mut first_chunk)?;
            let first_chunk = u32::from_be_bytes(first_chunk);

            let mut samples_per_chunk = [0u8; 4];
            c.read_exact(&mut samples_per_chunk)?;
            let samples_per_chunk = u32::from_be_bytes(samples_per_chunk);

            let mut sample_description_index = [0u8; 4];
            c.read_exact(&mut sample_description_index)?;
            let sample_description_index = u32::from_be_bytes(sample_description_index);

            entries.push((first_chunk, samples_per_chunk, sample_description_index));
        }

        let stsc = Stsc {
            version: version[0],
            flags,
            entries,
        };

        info!("stsc: {stsc:?}");

        Ok(stsc)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stsz {
    pub version: u8,
    pub flags: [u8; 3],
    pub sample_size: u32,
    pub sample_count: u32,
    pub entries: Vec<u32>,
}

impl Stsz {
    #[tracing::instrument(skip_all, name = "stsz")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Stsz> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut sample_size = [0u8; 4];
        c.read_exact(&mut sample_size)?;
        let sample_size = u32::from_be_bytes(sample_size);

        let mut sample_count = [0u8; 4];
        c.read_exact(&mut sample_count)?;
        let sample_count = u32::from_be_bytes(sample_count);

        let mut entries = Vec::new();
        if sample_size == 0 {
            for _ in 0..sample_count {
                let mut entry_size = [0u8; 4];
                c.read_exact(&mut entry_size)?;
                entries.push(u32::from_be_bytes(entry_size));
            }
        }

        let stsz = Stsz {
            version: version[0],
            flags,
            sample_size,
            sample_count,
            entries,
        };

        info!("stsz: {stsz:?}");

        Ok(stsz)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stbl {
    pub stsd: Stsd,
    pub stts: Stts,
    pub stss: Option<Stss>,
    pub stsc: Stsc,
    pub stsz: Stsz,
}

impl Stbl {
    #[tracing::instrument(skip_all, name = "stbl")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Stbl> {
        let mut stsd = None;
        let mut stts = None;
        let mut stss = None;
        let mut stsc = None;
        let mut stsz = None;
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
                "stss" => stss = Some(Stss::new(c)?),
                "stsc" => stsc = Some(Stsc::new(c)?),
                "stsz" => stsz = Some(Stsz::new(c)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        let stbl = Stbl {
            stsd: stsd.context("no stsd found")?,
            stts: stts.context("no stts found")?,
            stss,
            stsc: stsc.context("no stsc found")?,
            stsz: stsz.context("no stsz found")?,
        };

        info!("stbl: {stbl:?}");

        Ok(stbl)
    }
}
