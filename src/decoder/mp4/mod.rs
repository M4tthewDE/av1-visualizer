use std::{
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::{bail, Result};
use tracing::info;

use self::ftyp::Ftyp;

mod ftyp;

// reference:
// https://github.com/alfg/mp4/blob/master/atom/box.go#L99

#[derive(Clone, Debug, Default)]
pub struct Mp4 {
    ftyp: Ftyp,
}

impl Mp4 {
    #[tracing::instrument(skip_all)]
    pub fn new(p: PathBuf) -> Result<Mp4> {
        let data = std::fs::read(p)?;
        info!("loaded {} bytes", data.len());

        let mut mp4 = Mp4::default();
        mp4.parse(data)?;
        Ok(mp4)
    }

    #[tracing::instrument(skip_all)]
    fn parse(&mut self, data: Vec<u8>) -> Result<()> {
        let size = data.len();
        let mut c = Cursor::new(data);
        loop {
            let mut box_size = [0u8; 4];
            c.read_exact(&mut box_size)?;
            let box_size = u32::from_be_bytes(box_size);

            let mut box_type = [0u8; 4];
            c.read_exact(&mut box_type)?;
            let box_type = String::from_utf8(box_type.to_vec())?;

            match box_type.as_str() {
                "ftyp" => self.ftyp = ftyp::ftyp(&mut c, box_size as usize)?,
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        Ok(())
    }
}
