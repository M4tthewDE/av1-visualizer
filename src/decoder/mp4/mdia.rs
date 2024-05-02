use anyhow::{bail, Context};
use std::io::{Cursor, Read};
use tracing::warn;

use anyhow::Result;

use super::{hdlr::Hdlr, mdhd::Mdhd, minf::Minf};

#[derive(Clone, Debug, Default)]
pub struct Mdia {
    pub mdhd: Mdhd,
    pub hdlr: Hdlr,
    pub minf: Minf,
}

impl Mdia {
    #[tracing::instrument(skip_all, name = "mdia")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Mdia> {
        // subtract 8 from start because we already parsed box_size and box_type
        let start = c.position() - 8;
        let mut mdhd = None;
        let mut hdlr = None;
        let mut minf = None;
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            // subtracting 8 bytes because box_size and box_type belong to the overall box size
            let box_size = u32::from_be_bytes(box_size) - 8;

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "mdhd" => mdhd = Some(Mdhd::new(c)?),
                "hdlr" => hdlr = Some(Hdlr::new(c)?),
                "minf" => minf = Some(Minf::new(c, box_size as usize)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == start + size as u64 {
                break;
            }
        }

        Ok(Mdia {
            mdhd: mdhd.context("no mdhd found")?,
            hdlr: hdlr.context("no hdlr found")?,
            minf: minf.context("no minf found")?,
        })
    }
}
