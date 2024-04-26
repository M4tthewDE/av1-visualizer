use anyhow::{bail, Context};
use std::io::{Cursor, Read};

use anyhow::Result;

use super::{dinf::Dinf, vmhd::Vmhd};

#[derive(Clone, Debug, Default)]
pub struct Minf {
    pub vmhd: Vmhd,
    pub dinf: Dinf,
}

impl Minf {
    #[tracing::instrument(skip_all, name = "minf")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Minf> {
        let mut vmhd = None;
        let mut dinf = None;
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let _box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "dinf" => dinf = Some(Dinf::new(c)?),
                "vmhd" => vmhd = Some(Vmhd::new(c)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        Ok(Minf {
            vmhd: vmhd.context("no mvhd found")?,
            dinf: dinf.context("no dinf found")?,
        })
    }
}
