use anyhow::{bail, Context, Result};
use std::io::{Cursor, Read};

use super::meta::Meta;

#[derive(Clone, Debug, Default)]
pub struct Udta {
    pub meta: Meta,
    pub chpl: Option<Vec<u8>>,
}

impl Udta {
    #[tracing::instrument(skip_all, name = "udta")]
    pub fn new(c: &mut Cursor<Vec<u8>>, start: u64, size: u32) -> Result<Udta> {
        let mut meta = None;
        let mut chpl = None;
        loop {
            let box_start = c.position();

            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "meta" => meta = Some(Meta::new(c, box_start, box_size)?),
                "chpl" => {
                    let mut data = vec![0u8; box_size as usize - 8];
                    c.read_exact(&mut data)?;
                    chpl = Some(data.to_vec())
                }
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == start + size as u64 {
                break;
            }
        }

        Ok(Udta {
            meta: meta.context("no meta found")?,
            chpl,
        })
    }
}
