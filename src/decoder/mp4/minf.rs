use anyhow::{bail, Context};
use std::io::{Cursor, Read};

use anyhow::Result;

use super::{dinf::Dinf, stbl::Stbl, vmhd::Vmhd};

#[derive(Clone, Debug, Default)]
pub struct Minf {
    pub vmhd: Vmhd,
    pub dinf: Dinf,
    pub stbl: Stbl,
}

impl Minf {
    #[tracing::instrument(skip_all, name = "minf")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Minf> {
        let mut vmhd = None;
        let mut dinf = None;
        let mut stbl = None;
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            // subtracting 8 bytes because box_size and box_type belong to the overall box size
            let box_size = u32::from_be_bytes(box_size) - 8;

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "dinf" => dinf = Some(Dinf::new(c)?),
                "vmhd" => vmhd = Some(Vmhd::new(c)?),
                "stbl" => stbl = Some(Stbl::new(c, box_size as usize)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        Ok(Minf {
            vmhd: vmhd.context("no mvhd found")?,
            dinf: dinf.context("no dinf found")?,
            stbl: stbl.context("no stbl found")?,
        })
    }
}
