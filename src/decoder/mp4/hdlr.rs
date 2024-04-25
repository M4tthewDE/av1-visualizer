use anyhow::bail;
use std::io::{BufRead, Cursor, Read};
use tracing::info;

use anyhow::Result;

#[derive(Clone, Debug, Default)]
pub struct Hdlr {
    pub version: u8,
    pub flags: [u8; 3],
    pub handler_type: String,
    pub name: String,
}

impl Hdlr {
    #[tracing::instrument(skip_all, name = "hdlr")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Hdlr> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        let mut pre_defined = [0u8; 4];
        c.read_exact(&mut pre_defined)?;
        let pre_defined = u32::from_be_bytes(pre_defined);
        if pre_defined != 0 {
            bail!("pre_defined has to be 0, is {pre_defined}");
        }

        let mut handler_type = [0u8; 4];
        c.read_exact(&mut handler_type)?;
        let handler_type = String::from_utf8(handler_type.to_vec())?;

        let mut reserved = [0u8; 12];
        c.read_exact(&mut reserved)?;
        for r in reserved {
            if r != 0 {
                bail!("reserved has to be 0, is {r}");
            }
        }

        let mut name = Vec::new();
        c.read_until(b'\0', &mut name)?;
        name.remove(name.len() - 1);

        let hdlr = Hdlr {
            version: version[0],
            flags,
            handler_type,
            name: String::from_utf8(name)?,
        };
        info!("hdlr: {hdlr:?}");

        Ok(hdlr)
    }
}
