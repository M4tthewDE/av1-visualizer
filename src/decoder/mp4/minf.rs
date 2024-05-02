use anyhow::{bail, Context};
use std::io::{Cursor, Read};

use anyhow::Result;

use super::{dinf::Dinf, gmhd::Gmhd, stbl::Stbl, vmhd::Vmhd};

#[derive(Clone, Debug)]
pub enum InformationHeader {
    Vmhd(Vmhd),
    Gmhd(Gmhd),
}

impl Default for InformationHeader {
    fn default() -> Self {
        InformationHeader::Vmhd(Vmhd::default())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Minf {
    pub information_header: InformationHeader,
    pub dinf: Dinf,
    pub stbl: Stbl,
}

impl Minf {
    #[tracing::instrument(skip_all, name = "minf")]
    pub fn new(c: &mut Cursor<Vec<u8>>, start: u64, size: u32) -> Result<Minf> {
        let mut information_header = None;
        let mut dinf = None;
        let mut stbl = None;
        loop {
            let box_start = c.position();

            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "dinf" => dinf = Some(Dinf::new(c)?),
                "vmhd" => information_header = Some(InformationHeader::Vmhd(Vmhd::new(c)?)),
                "gmhd" => {
                    information_header = Some(InformationHeader::Gmhd(Gmhd::new(c, box_size)?));
                }
                "stbl" => stbl = Some(Stbl::new(c, box_start, box_size)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == start + size as u64 {
                break;
            }
        }

        Ok(Minf {
            information_header: information_header.context("no information header found")?,
            dinf: dinf.context("no dinf found")?,
            stbl: stbl.context("no stbl found")?,
        })
    }
}
