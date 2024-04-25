use anyhow::{Context, Result};
use std::io::{Cursor, Read};

use anyhow::bail;

use super::{
    mvhd::{self, Mvhd},
    trak::Trak,
};

#[derive(Clone, Debug, Default)]
pub struct Moov {
    pub mvhd: Mvhd,
    pub traks: Vec<Trak>,
}

#[tracing::instrument(skip_all)]
pub fn moov(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Moov> {
    let mut mvhd = None;
    let mut traks = Vec::new();

    loop {
        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;
        let box_size = u32::from_be_bytes(box_size);

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;

        match box_type.as_str() {
            "mvhd" => mvhd = Some(mvhd::parse_mvhd(c)?),
            "trak" => traks.push(Trak::new(c, box_size as usize)?),
            typ => bail!("box type {typ:?} not implemented"),
        }

        if c.position() == size as u64 {
            break;
        }
    }

    if traks.is_empty() {
        bail!("at least one trak required in moov");
    }

    Ok(Moov {
        mvhd: mvhd.context("no mvhd found")?,
        traks,
    })
}
