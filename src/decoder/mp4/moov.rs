use anyhow::{Context, Result};
use std::io::{Cursor, Read};

use anyhow::bail;

use super::{mvhd::Mvhd, trak::Trak, udta::Udta};

/// The metadata for a presentation is stored in the single Movie Box which occurs at the top‐level of a file.
/// Normally this box is close to the beginning or end of the file, though this is not required.
///
/// Box Type: 'moov'
/// Mandatory: Yes
/// Quantity: Exactly one
#[derive(Clone, Debug, Default)]
pub struct Moov {
    pub mvhd: Mvhd,
    pub traks: Vec<Trak>,
    pub udta: Option<Udta>,
    pub mdat: Option<Vec<u8>>,
}

impl Moov {
    #[tracing::instrument(skip_all, name = "moov")]
    pub fn new(c: &mut Cursor<Vec<u8>>, start: u64, size: u32) -> Result<Moov> {
        let mut mvhd = None;
        let mut traks = Vec::new();
        let mut udta = None;
        let mut mdat = None;

        loop {
            let box_start = c.position();
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "mvhd" => mvhd = Some(Mvhd::new(c)?),
                "trak" => traks.push(Trak::new(c, box_start, box_size)?),
                "udta" => udta = Some(Udta::new(c, box_start, box_size)?),
                "mdat" => {
                    let mut data = vec![0u8; box_size as usize];
                    c.read_exact(&mut data)?;
                    mdat = Some(data.to_vec())
                }
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == start + size as u64 {
                break;
            }
        }

        if traks.is_empty() {
            bail!("at least one trak required in moov");
        }

        Ok(Moov {
            mvhd: mvhd.context("no mvhd found")?,
            traks,
            udta,
            mdat,
        })
    }
}
