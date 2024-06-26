use anyhow::Result;
use obu::{SequenceHeader, TxMode, UncompressedHeader, WarpModel};

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

    fn alignment(&mut self) {
        while (self.pos & 7) != 0 {
            self.f(1);
        }
    }

    fn le(&mut self, n: u64) -> u64 {
        let mut t = 0;
        for i in 0..n {
            t += self.f(8) << (i * 8);
        }

        t
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

#[derive(Debug, Clone, Copy)]
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
    pub tile_num: u64,
    pub uh: UncompressedHeader,
    pub num_tiles: u64,
    pub mi_row_start: u64,
    pub mi_row_end: u64,
    pub mi_col_start: u64,
    pub mi_col_end: u64,
    pub symbol_range: u64,
    pub symbol_max_bits: i64,
    pub above_level_context: Vec<Vec<u64>>,
    pub above_dc_context: Vec<Vec<u64>>,
    pub above_seg_pred_context: Vec<u64>,
    pub delta_lf: Vec<u64>,
    pub ref_sgr_xqd: Vec<Vec<i64>>,
    pub ref_lr_wiener: Vec<Vec<Vec<i64>>>,
    pub left_level_context: Vec<Vec<u64>>,
    pub left_dc_context: Vec<Vec<u64>>,
    pub left_seg_pred_context: Vec<u64>,
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
