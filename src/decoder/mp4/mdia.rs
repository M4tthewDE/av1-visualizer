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
    pub fn new(c: &mut Cursor<Vec<u8>>, start: u64, size: u32) -> Result<Mdia> {
        let mut mdhd = None;
        let mut hdlr = None;
        let mut minf = None;
        loop {
            let box_start = c.position();

            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "mdhd" => mdhd = Some(Mdhd::new(c)?),
                "hdlr" => hdlr = Some(Hdlr::new(c)?),
                "minf" => minf = Some(Minf::new(c, box_start, box_size)?),
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
