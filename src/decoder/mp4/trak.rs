use std::io::{Cursor, Read};

use anyhow::{bail, Context, Result};

use super::{edts::Edts, mdia::Mdia, tkhd::Tkhd, tref::Tref};

#[derive(Clone, Debug, Default)]
pub struct Trak {
    pub tkhd: Tkhd,
    pub edts: Option<Edts>,
    pub tref: Option<Tref>,
    pub mdia: Mdia,
}

impl Trak {
    #[tracing::instrument(skip_all, name = "trak")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Trak> {
        let mut tkhd = None;
        let mut edts = None;
        let mut tref = None;
        let mut mdia = None;
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "tkhd" => tkhd = Some(Tkhd::new(c)?),
                "edts" => edts = Some(Edts::new(c)?),
                "tref" => tref = Some(Tref::new(c, box_size as usize)?),
                "mdia" => mdia = Some(Mdia::new(c, box_size as usize)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        Ok(Trak {
            tkhd: tkhd.context("no tkhd found")?,
            edts,
            tref,
            mdia: mdia.context("no tkhd found")?,
        })
    }
}
