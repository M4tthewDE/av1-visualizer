use std::{
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::{bail, Result};
use tracing::info;

use self::{ftyp::Ftyp, moov::Moov};

mod av01;
mod av1c;
mod dinf;
mod dref;
mod edts;
mod elst;
mod ftyp;
mod hdlr;
mod mdhd;
mod mdia;
mod minf;
mod moov;
mod mvhd;
mod stbl;
mod stsd;
mod tkhd;
mod trak;
mod tref;
mod vmhd;

// reference:
// https://b.goeswhere.com/ISO_IEC_14496-12_2015.pdf

#[derive(Clone, Debug, Default)]
pub struct Mp4 {
    ftyp: Ftyp,
    moov: Moov,
}

impl Mp4 {
    #[tracing::instrument(skip_all, name = "mp4")]
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
                "ftyp" => self.ftyp = Ftyp::new(&mut c, box_size as usize)?,
                "moov" => self.moov = Moov::new(&mut c, box_size as usize)?,
                typ => bail!("box type {typ:?} not implemented"),
            }

            if c.position() == size as u64 {
                break;
            }
        }

        Ok(())
    }
}

fn fixed32(data: [u8; 4]) -> f64 {
    u32::from_be_bytes(data) as f64 / 65536.0
}

fn fixed16(data: [u8; 2]) -> f64 {
    u16::from_be_bytes(data) as f64 / 256.0
}
