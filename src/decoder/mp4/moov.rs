use anyhow::{Context, Result};
use std::io::{Cursor, Read};

use anyhow::bail;

use super::{
    mvhd::{self, Mvhd},
    tkhd::{self, Tkhd},
};

#[derive(Clone, Debug, Default)]
pub struct Moov {
    pub mvhd: Mvhd,
    pub tkhd: Tkhd,
}

#[tracing::instrument(skip_all)]
pub fn moov(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Moov> {
    let mut mvhd = None;
    let mut tkhd = None;

    loop {
        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;
        let box_size = u32::from_be_bytes(box_size);

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;

        match box_type.as_str() {
            "mvhd" => mvhd = Some(mvhd::parse_mvhd(c, box_size as u64)?),
            "tkhd" => tkhd = Some(tkhd::parse_tkhd(c)?),
            typ => bail!("box type {typ:?} not implemented"),
        }

        if c.position() == size as u64 {
            break;
        }
    }

    Ok(Moov {
        mvhd: mvhd.context("no mvhd found")?,
        tkhd: tkhd.context("no tkhd found")?,
    })
}
