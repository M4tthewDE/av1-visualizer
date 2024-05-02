use anyhow::bail;
use std::io::{Cursor, Read};
use tracing::info;

use anyhow::Result;

use super::av01::Av01;

#[derive(Clone, Debug, Default)]
pub struct SampleEntry {
    pub format: String,
    pub data_reference_index: u16,
    pub av01: Av01,
}

#[derive(Clone, Debug, Default)]
pub struct Stsd {
    pub version: u8,
    pub flags: [u8; 3],
    pub entry_count: u32,

    /// Unsure what this is used for
    pub handler_type: u32,
    pub sample_entries: Vec<SampleEntry>,
}

impl Stsd {
    #[tracing::instrument(skip_all, name = "stsd")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Stsd> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut entry_count = [0u8; 4];
        c.read_exact(&mut entry_count)?;
        let entry_count = u32::from_be_bytes(entry_count);

        let mut handler_type = [0u8; 4];
        c.read_exact(&mut handler_type)?;
        let handler_type = u32::from_be_bytes(handler_type);

        let mut sample_entries = Vec::new();
        for _ in 0..entry_count {
            let mut format = [0u8; 4];
            c.read_exact(&mut format)?;
            let format = String::from_utf8(format.to_vec())?;

            if format != "av01" {
                bail!("sample format {format} is not supported");
            }

            // reserved
            c.set_position(c.position() + 6);

            let mut data_reference_index = [0u8; 2];
            c.read_exact(&mut data_reference_index)?;
            let data_reference_index = u16::from_be_bytes(data_reference_index);

            let av01 = Av01::new(c)?;

            sample_entries.push(SampleEntry {
                format,
                data_reference_index,
                av01,
            });
        }

        Ok(Stsd {
            version: version[0],
            flags,
            entry_count,
            handler_type,
            sample_entries,
        })
    }
}
