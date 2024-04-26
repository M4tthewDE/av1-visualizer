use anyhow::{bail, Context};
use std::io::{Cursor, Read};

use anyhow::Result;

use super::stsd::Stsd;

#[derive(Clone, Debug, Default)]
pub struct Stbl {
    pub stsd: Stsd,
}

impl Stbl {
    #[tracing::instrument(skip_all, name = "stbl")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Stbl> {
        let mut stsd = None;
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let _box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "stsd" => stsd = Some(Stsd::new(c)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        Ok(Stbl {
            stsd: stsd.context("no stsd found")?,
        })
    }
}
