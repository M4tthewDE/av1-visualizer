use anyhow::bail;
use std::io::{Cursor, Read};

use anyhow::Result;

use super::elst::Elst;

#[derive(Clone, Debug, Default)]
pub struct Edts {
    pub elst: Option<Elst>,
}

impl Edts {
    #[tracing::instrument(skip_all)]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Edts> {
        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;
        let _box_size = u32::from_be_bytes(box_size);

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;

        match box_type.as_str() {
            "elst" => {
                return Ok(Edts {
                    elst: Some(Elst::new(c)?),
                })
            }
            typ => bail!("box type {typ:?} not implemented"),
        }
    }
}
