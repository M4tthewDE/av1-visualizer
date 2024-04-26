use anyhow::bail;
use std::io::{Cursor, Read};

use anyhow::Result;

use super::dref::Dref;

#[derive(Clone, Debug, Default)]
pub struct Dinf {
    pub dref: Dref,
}

impl Dinf {
    #[tracing::instrument(skip_all, name = "dinf")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Dinf> {
        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;
        let box_size = u32::from_be_bytes(box_size);

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;

        match box_type.as_str() {
            "dref" => Ok(Dinf {
                dref: Dref::new(c, box_size as usize)?,
            }),
            typ => bail!("box type {typ:?} not implemented"),
        }
    }
}
