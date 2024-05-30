use anyhow::Result;
use std::fmt::Display;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use tracing::info;

// https://formats.kaitai.io/vp8_ivf/

#[derive(Debug)]
pub struct Ivf {
    pub header_length: u16,
    pub fourcc: String,
    pub width: u16,
    pub height: u16,
    pub denominator: u32,
    pub numerator: u32,
    pub num_frames: u32,
    pub blocks: Vec<Block>,
}

impl Display for Ivf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "header_length: {}", self.header_length)?;
        writeln!(f, "fourcc: {}", self.fourcc)?;
        writeln!(f, "width: {}", self.width)?;
        writeln!(f, "height: {}", self.height)?;
        writeln!(f, "denominator: {}", self.denominator)?;
        writeln!(f, "numerator: {}", self.numerator)?;
        write!(f, "num_frame: {}", self.num_frames)
    }
}

impl Ivf {
    #[tracing::instrument(skip_all, name = "ivf")]
    pub fn new(p: PathBuf) -> Result<Ivf> {
        let data = std::fs::read(p)?;
        info!("loaded {} bytes", data.len());

        let mut c = Cursor::new(data);

        let mut signature = [0u8; 4];
        c.read_exact(&mut signature)?;
        let signature = String::from_utf8(signature.to_vec())?;
        assert_eq!(signature, "DKIF");

        let mut version = [0u8; 2];
        c.read_exact(&mut version)?;
        let version = u16::from_be_bytes(version);
        assert_eq!(version, 0);

        let mut header_length = [0u8; 2];
        c.read_exact(&mut header_length)?;
        let header_length = u16::from_le_bytes(header_length);

        let mut fourcc = [0u8; 4];
        c.read_exact(&mut fourcc)?;
        let fourcc = String::from_utf8(fourcc.to_vec())?;

        let mut width = [0u8; 2];
        c.read_exact(&mut width)?;
        let width = u16::from_le_bytes(width);

        let mut height = [0u8; 2];
        c.read_exact(&mut height)?;
        let height = u16::from_le_bytes(height);

        let mut denominator = [0u8; 4];
        c.read_exact(&mut denominator)?;
        let denominator = u32::from_le_bytes(denominator);

        let mut numerator = [0u8; 4];
        c.read_exact(&mut numerator)?;
        let numerator = u32::from_le_bytes(numerator);

        let mut num_frames = [0u8; 4];
        c.read_exact(&mut num_frames)?;
        let num_frames = u32::from_le_bytes(num_frames);

        c.set_position(c.position() + 4);

        let mut blocks = Vec::new();
        for _ in 0..num_frames {
            blocks.push(Block::new(&mut c)?);
        }

        return Ok(Ivf {
            header_length,
            fourcc,
            width,
            height,
            denominator,
            numerator,
            num_frames,
            blocks,
        });
    }
}

#[derive(Debug)]
pub struct Block {
    pub len_frame: u32,
    pub timestamp: u64,
    pub framedata: Vec<u8>,
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "len_frame: {}, timestamp: {}",
            self.len_frame, self.timestamp
        )
    }
}

impl Block {
    #[tracing::instrument(skip_all, name = "ivf")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Block> {
        let mut len_frame = [0u8; 4];
        c.read_exact(&mut len_frame)?;
        let len_frame = u32::from_le_bytes(len_frame);

        let mut timestamp = [0u8; 8];
        c.read_exact(&mut timestamp)?;
        let timestamp = u64::from_le_bytes(timestamp);

        let mut framedata = vec![0u8; len_frame as usize];
        c.read_exact(&mut framedata)?;

        Ok(Block {
            len_frame,
            timestamp,
            framedata,
        })
    }
}
