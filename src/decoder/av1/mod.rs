use anyhow::Result;
use tracing::info;

use super::ivf::Ivf;

mod obu;

#[derive(Debug)]
pub struct BitStream {
    pos: usize,
    data: Vec<u8>,

    seen_frame_header: bool,
}

impl BitStream {
    pub fn new(data: Vec<u8>) -> BitStream {
        BitStream {
            pos: 0,
            data,
            seen_frame_header: false,
        }
    }

    fn read_bit(&mut self) -> u8 {
        let res = (self.data[self.pos / 8] >> (7 - self.pos % 8)) & 1;
        self.pos += 1;
        res
    }

    fn f(self: &mut BitStream, n: u64) -> u64 {
        let mut x: u64 = 0;
        for _ in 0..n {
            x = 2 * x + self.read_bit() as u64;
        }

        x
    }

    fn leb128(self: &mut BitStream) -> u64 {
        let mut value = 0;

        for i in 0..8 {
            let leb128_byte = self.f(8);
            value |= (leb128_byte & 0x7f) << (i * 7);

            if (leb128_byte & 0x80) == 0 {
                break;
            }
        }

        value
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BitDepth {
    Invalid = -1,
    Eight = 8,
    Ten = 10,
    Twelve = 12,
}

impl Default for BitDepth {
    fn default() -> Self {
        Self::Invalid
    }
}

#[derive(Debug)]
pub enum NumPlanes {
    One = 1,
    Three = 3,
}

impl Default for NumPlanes {
    fn default() -> Self {
        Self::One
    }
}

#[derive(Debug, Default)]
pub struct Decoder {
    pub bit_depth: BitDepth,
    pub num_planes: NumPlanes,
    pub order_hint_bits: u64,
}

impl Decoder {
    pub fn decode(&mut self, ivf: Ivf) -> Result<()> {
        for block in &ivf.blocks {
            let mut b = BitStream::new(block.framedata.clone());
            loop {
                let obu = self.obu(&mut b);
                info!("obu: {:?}", obu);
            }
        }

        Ok(())
    }
}
