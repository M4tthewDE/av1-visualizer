use anyhow::{bail, Context, Result};
use std::io::{Cursor, Read};

use super::hdlr::Hdlr;

#[derive(Clone, Debug, Default)]
pub struct Meta {
    pub hdlr: Hdlr,
}

impl Meta {
    #[tracing::instrument(skip_all, name = "meta")]
    pub fn new(c: &mut Cursor<Vec<u8>>, start: u64, size: u32) -> Result<Meta> {
        let mut hdlr = None;

        // FIXME: why do we need to skip 4 bytes here?
        c.set_position(c.position() + 4);
        loop {
            let _box_start = c.position();

            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let _box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "hdlr" => hdlr = Some(Hdlr::new(c)?),
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == start + size as u64 {
                break;
            }
        }

        dbg!("META");
        Ok(Meta {
            hdlr: hdlr.context("no hdlr found")?,
        })
    }
}
