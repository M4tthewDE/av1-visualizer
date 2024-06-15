use anyhow::Result;
use obu::{SequenceHeader, TxMode, WarpModel};

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

    fn f(&mut self, n: u64) -> u64 {
        let mut x: u64 = 0;
        for _ in 0..n {
            x = 2 * x + self.read_bit() as u64;
        }

        x
    }

    fn leb128(&mut self) -> u64 {
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

    fn su(&mut self, n: u64) -> i64 {
        let value = self.f(n) as i64;
        let sign_mask = 1 << (n - 1);
        if (value & sign_mask) != 0 {
            value - 2 * sign_mask
        } else {
            value
        }
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
    pub seen_frame_header: bool,
    pub sequence_header: SequenceHeader,
    pub frame_is_intra: bool,
    pub ref_valid: [bool; 8],
    pub ref_order_hint: [bool; 8],
    pub order_hint: u64,
    pub frame_width: u64,
    pub frame_height: u64,
    pub superres_denom: u64,
    pub upscaled_width: u64,
    pub mi_cols: u64,
    pub mi_rows: u64,
    pub render_width: u64,
    pub render_height: u64,
    pub tile_cols_log2: u64,
    pub mi_col_starts: Vec<u64>,
    pub tile_cols: u64,
    pub tile_rows_log2: u64,
    pub mi_row_starts: Vec<u64>,
    pub tile_rows: u64,
    pub tile_size_bytes: u64,
    pub deltaq_ydc: i64,
    pub deltaq_udc: i64,
    pub deltaq_uac: i64,
    pub deltaq_vdc: i64,
    pub deltaq_vac: i64,
    pub feature_enabled: Vec<Vec<bool>>,
    pub feature_data: Vec<Vec<u64>>,
    pub seg_id_pre_skip: bool,
    pub last_active_seg_id: u64,
    pub coded_lossless: bool,
    pub current_q_index: u64,
    pub lossless_array: Vec<bool>,
    pub all_lossless: bool,
    pub cdef_damping: u64,
    pub frame_restoration_type: Vec<u64>,
    pub uses_lr: bool,
    pub tx_mode: TxMode,
    pub gm_type: [WarpModel; 8],
    pub order_hints: [bool; Decoder::REFS_PER_FRAME as usize + Decoder::LAST_FRAME],
}

impl Decoder {
    pub fn decode(&mut self, ivf: Ivf) -> Result<()> {
        for block in &ivf.blocks {
            let mut b = BitStream::new(block.framedata.clone());
            self.decode_frame(&mut b);
        }

        Ok(())
    }

    fn decode_frame(&mut self, b: &mut BitStream) {
        loop {
            self.obu(b);
        }
    }
}
