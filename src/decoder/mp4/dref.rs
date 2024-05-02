use std::io::BufRead;
use std::io::{Cursor, Read};

use anyhow::{bail, Result};

#[derive(Clone, Debug)]
pub enum DataEntry {
    Url {
        version: u8,
        flags: [u8; 3],
        location: String,
    },
    Urn {
        version: u8,
        flags: [u8; 3],
        name: String,
        location: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Dref {
    pub version: u8,
    pub flags: [u8; 3],
    pub entry_count: u32,
    pub entries: Vec<DataEntry>,
}

impl Dref {
    #[tracing::instrument(skip_all, name = "dref")]
    pub fn new(c: &mut Cursor<Vec<u8>>, start: u64, size: u32) -> Result<Dref> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut entry_count = [0u8; 4];
        c.read_exact(&mut entry_count)?;
        let entry_count = u32::from_be_bytes(entry_count);

        let mut entries = Vec::new();
        for _ in 0..entry_count {
            let mut version = [0u8; 1];
            c.read_exact(&mut version)?;

            let mut flags = [0u8; 3];
            c.read_exact(&mut flags)?;

            let mut entry_type = [0u8; 4];
            c.read_exact(&mut entry_type)?;
            match String::from_utf8(entry_type.to_vec())?.as_str() {
                "url " => {
                    let mut location = Vec::new();
                    c.read_until(b'\0', &mut location)?;
                    location.remove(location.len() - 1);
                    let location = String::from_utf8(location.to_vec())?;
                    entries.push(DataEntry::Url {
                        version: version[0],
                        flags,
                        location,
                    });
                }
                "urn " => {
                    let mut name = Vec::new();
                    c.read_until(b'\0', &mut name)?;
                    name.remove(name.len() - 1);
                    let name = String::from_utf8(name.to_vec())?;

                    let mut location = Vec::new();
                    c.read_until(b'\0', &mut location)?;
                    location.remove(location.len() - 1);
                    let location = String::from_utf8(location.to_vec())?;

                    entries.push(DataEntry::Urn {
                        version: version[0],
                        flags,
                        name,
                        location,
                    });
                }
                e => bail!("unknown entry_type {e}"),
            }
        }

        c.set_position(start + size as u64);

        Ok(Dref {
            version: version[0],
            flags,
            entry_count,
            entries,
        })
    }
}
