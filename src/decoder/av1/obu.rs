use tracing::info;

use super::{BitDepth, BitStream, Decoder, NumPlanes};

#[derive(Debug, Clone)]
pub enum ObuType {
    SequenceHeader,
    TemporalDelimiter,
    TileGroup,
    TileList,
    Frame,
}

impl ObuType {
    fn new(val: u64) -> ObuType {
        match val {
            1 => ObuType::SequenceHeader,
            2 => ObuType::TemporalDelimiter,
            4 => ObuType::TileGroup,
            6 => ObuType::Frame,
            8 => ObuType::TileList,
            v => panic!("unknown obu type: {v}"),
        }
    }
}

#[derive(Debug)]
pub struct ObuHeader {
    pub obu_type: ObuType,
    pub has_size: bool,
}

impl ObuHeader {
    pub fn new(b: &mut BitStream) -> ObuHeader {
        let forbidden_bit = b.f(1);
        assert_eq!(forbidden_bit, 0);

        let obu_type = ObuType::new(b.f(4));
        let extension_flag = b.f(1) != 0;
        let has_size = b.f(1) != 0;
        let _reserved_bit = b.f(1);

        if extension_flag {
            todo!("parse extension header");
        }

        ObuHeader { obu_type, has_size }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SeqProfile {
    Zero = 0,
    One = 1,
    Two = 2,
}

impl SeqProfile {
    fn new(val: u64) -> Self {
        match val {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            _ => panic!("invalid seq_profile: {val}"),
        }
    }
}

#[derive(Debug)]
pub enum Obu {
    TemporalDelimiter {
        header: ObuHeader,
    },
    SequenceHeader {
        header: ObuHeader,
        still_picture: bool,
        timing_info_present: bool,
        decoder_model_info_present: bool,
        initial_display_delay_present: bool,
        operating_points_cnt: u64,
        operating_point_idc: Vec<u64>,
        seq_level_idx: Vec<u64>,
        seq_tier: Vec<u64>,
        decoder_model_present_for_this_op: Vec<bool>,
        initial_display_delay_present_for_this_op: Vec<bool>,
        initial_display_delay: Vec<u64>,
        max_frame_width: u64,
        max_frame_height: u64,
        use_128x128_superblock: bool,
        enable_filter_intra: bool,
        enable_intra_edge_filter: bool,
        enable_interintra_compound: bool,
        enable_masked_compound: bool,
        enable_warped_motion: bool,
        enable_dual_filter: bool,
        enable_order_hint: bool,
        enable_jnt_comp: bool,
        enable_ref_frame_mvs: bool,
        seq_force_integer_mv: u64,
        enable_superres: bool,
        enable_cdef: bool,
        enable_restoration: bool,
        color_config: ColorConfig,
        film_grain_params_present: bool,
    },
}

impl Decoder {
    pub fn obu(&mut self, b: &mut BitStream) -> Obu {
        let header = ObuHeader::new(b);
        let size = if header.has_size {
            b.leb128()
        } else {
            todo!("where does sz come from?");
        };

        info!("size: {size}");

        let start_position = b.pos;

        let obu_type = header.obu_type.clone();

        let obu = match obu_type {
            ObuType::SequenceHeader => self.sequence_header(b, header),
            ObuType::TemporalDelimiter => {
                b.seen_frame_header = false;
                Obu::TemporalDelimiter { header }
            }
            t => panic!("obu type not implemented: {t:?}"),
        };

        let current_position = b.pos;
        let payload_bits = current_position - start_position;

        if size > 0
            && !matches!(obu_type, ObuType::TileGroup)
            && !matches!(obu_type, ObuType::TileList)
            && !matches!(obu_type, ObuType::Frame)
        {
            let mut nb_bits = size * 8 - payload_bits as u64;
            b.f(1);
            nb_bits -= 1;

            while nb_bits > 0 {
                b.f(1);
                nb_bits -= 1;
            }
        }

        obu
    }

    const SELECT_INTEGER_MV: u64 = 2;

    fn sequence_header(&mut self, b: &mut BitStream, header: ObuHeader) -> Obu {
        let seq_profile = SeqProfile::new(b.f(3));
        let still_picture = b.f(1) != 0;
        let reduced_still_picture_header = b.f(1) != 0;

        let decoder_model_info_present: bool;
        let mut operating_point_idc: Vec<u64>;
        let mut seq_level_idx: Vec<u64>;
        let mut seq_tier: Vec<u64>;
        let decoder_model_present_for_this_op: Vec<bool>;
        let mut initial_display_delay_present_for_this_op: Vec<bool>;
        let mut initial_display_delay: Vec<u64> = Vec::new();
        let timing_info_present: bool;
        let initial_display_delay_present: bool;
        let operating_points_cnt: u64;

        if reduced_still_picture_header {
            timing_info_present = false;
            decoder_model_info_present = false;
            initial_display_delay_present = false;
            operating_points_cnt = 1;
            operating_point_idc = vec![0];
            seq_level_idx = vec![0];
            seq_tier = vec![0];
            decoder_model_present_for_this_op = vec![false];
            initial_display_delay_present_for_this_op = vec![false];
        } else {
            timing_info_present = b.f(1) != 0;
            if timing_info_present {
                todo!("timing_info_present_flag == true");
            } else {
                decoder_model_info_present = false;
            }

            initial_display_delay_present = b.f(1) != 0;
            operating_points_cnt = b.f(5) + 1;

            operating_point_idc = vec![0; operating_points_cnt as usize];
            seq_level_idx = vec![0; operating_points_cnt as usize];
            seq_tier = vec![0; operating_points_cnt as usize];
            decoder_model_present_for_this_op = vec![false; operating_points_cnt as usize];
            initial_display_delay_present_for_this_op = vec![false; operating_points_cnt as usize];
            initial_display_delay = vec![0; operating_points_cnt as usize];

            for i in 0..operating_points_cnt as usize {
                operating_point_idc[i] = b.f(12);
                seq_level_idx[i] = b.f(5);

                if seq_level_idx[i] > 7 {
                    seq_tier[i] = b.f(1);
                } else {
                    seq_tier[i] = 0;
                }

                if decoder_model_info_present {
                    todo!();
                }

                if initial_display_delay_present {
                    initial_display_delay_present_for_this_op[i] = b.f(1) != 0;
                    if initial_display_delay_present_for_this_op[i] {
                        initial_display_delay[i] = b.f(4) - 1;
                    }
                }
            }
        }

        let frame_width_bits = b.f(4) + 1;
        let frame_height_bits = b.f(4) + 1;
        let max_frame_width = b.f(frame_width_bits) + 1;
        let max_frame_height = b.f(frame_height_bits) + 1;

        let frame_id_numbers_present = if reduced_still_picture_header {
            false
        } else {
            b.f(1) != 0
        };

        if frame_id_numbers_present {
            todo!("frame_id_numbers_present == true");
        }

        let use_128x128_superblock = b.f(1) != 0;
        let enable_filter_intra = b.f(1) != 0;
        let enable_intra_edge_filter = b.f(1) != 0;

        let enable_interintra_compound: bool;
        let enable_masked_compound: bool;
        let enable_warped_motion: bool;
        let enable_dual_filter: bool;
        let enable_order_hint: bool;
        let enable_jnt_comp: bool;
        let enable_ref_frame_mvs: bool;
        let seq_force_integer_mv: u64;

        if reduced_still_picture_header {
            todo!();
        } else {
            enable_interintra_compound = b.f(1) != 0;
            enable_masked_compound = b.f(1) != 0;
            enable_warped_motion = b.f(1) != 0;
            enable_dual_filter = b.f(1) != 0;
            enable_order_hint = b.f(1) != 0;

            (enable_jnt_comp, enable_ref_frame_mvs) = if enable_order_hint {
                (b.f(1) != 0, b.f(1) != 0)
            } else {
                (false, false)
            };

            let seq_choose_screen_content_tools = b.f(1) != 0;
            let seq_force_screen_content_tools = if seq_choose_screen_content_tools {
                2
            } else {
                b.f(1)
            };

            seq_force_integer_mv = if seq_force_screen_content_tools > 0 {
                if b.f(1) != 0 {
                    Decoder::SELECT_INTEGER_MV
                } else {
                    b.f(1)
                }
            } else {
                2
            };

            self.order_hint_bits = if enable_order_hint { b.f(3) + 1 } else { 0 };
        }

        Obu::SequenceHeader {
            header,
            still_picture,
            timing_info_present,
            decoder_model_info_present,
            initial_display_delay_present,
            operating_points_cnt,
            operating_point_idc,
            seq_level_idx,
            seq_tier,
            decoder_model_present_for_this_op,
            initial_display_delay_present_for_this_op,
            initial_display_delay,
            max_frame_width,
            max_frame_height,
            use_128x128_superblock,
            enable_filter_intra,
            enable_intra_edge_filter,
            enable_interintra_compound,
            enable_masked_compound,
            enable_warped_motion,
            enable_dual_filter,
            enable_order_hint,
            enable_jnt_comp,
            enable_ref_frame_mvs,
            seq_force_integer_mv,
            enable_superres: b.f(1) != 0,
            enable_cdef: b.f(1) != 0,
            enable_restoration: b.f(1) != 0,
            color_config: self.color_config(b, seq_profile),
            film_grain_params_present: b.f(1) != 0,
        }
    }
}

#[derive(Debug)]
enum ColorPrimaries {
    Bt709 = 1,
    Unspecified = 2,
}

impl ColorPrimaries {
    fn new(val: u64) -> ColorPrimaries {
        match val {
            1 => ColorPrimaries::Bt709,
            2 => ColorPrimaries::Unspecified,
            _ => panic!("invalid value for ColorPrimaries: {val}"),
        }
    }
}

#[derive(Debug)]
enum TransferCharacteristics {
    Unspecified = 2,
    Srgb = 13,
}

impl TransferCharacteristics {
    fn new(val: u64) -> TransferCharacteristics {
        match val {
            2 => TransferCharacteristics::Unspecified,
            13 => TransferCharacteristics::Srgb,
            _ => panic!("invalid value for TransferCharacterstics: {val}"),
        }
    }
}

#[derive(Debug)]
enum MatrixCoefficients {
    Unspecified = 0,
    Identity = 2,
}

impl MatrixCoefficients {
    fn new(val: u64) -> MatrixCoefficients {
        match val {
            0 => MatrixCoefficients::Identity,
            2 => MatrixCoefficients::Unspecified,
            _ => panic!("invalid value for MatrixCoefficients: {val}"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ChromaSamplePosition {
    Unknown = 0,
    Vertical = 1,
    Colocated = 2,
    Reserved = 3,
}

impl ChromaSamplePosition {
    fn new(val: u64) -> Self {
        match val {
            0 => Self::Unknown,
            1 => Self::Vertical,
            2 => Self::Colocated,
            3 => Self::Reserved,
            _ => panic!("invalid seq_profile: {val}"),
        }
    }
}

#[derive(Debug)]
pub struct ColorConfig {
    pub separate_uv_delta_q: bool,
    pub color_range: bool,
    pub subsampling_x: bool,
    pub subsampling_y: bool,
    pub chroma_sample_position: ChromaSamplePosition,
}

impl Decoder {
    fn color_config(&mut self, b: &mut BitStream, seq_profile: SeqProfile) -> ColorConfig {
        let high_bitdepth = b.f(1) != 0;

        self.bit_depth = if seq_profile as u64 == 2 && high_bitdepth {
            if b.f(1) != 0 {
                BitDepth::Twelve
            } else {
                BitDepth::Ten
            }
        } else if seq_profile as u64 <= 2 && high_bitdepth {
            BitDepth::Ten
        } else {
            BitDepth::Eight
        };

        let monochrome = if seq_profile as u64 == 1 {
            false
        } else {
            b.f(1) != 0
        };

        self.num_planes = if monochrome {
            NumPlanes::One
        } else {
            NumPlanes::Three
        };

        let (color_primaries, transfer_characteristics, matrix_coefficients) = if b.f(1) != 0 {
            (
                ColorPrimaries::new(b.f(8)),
                TransferCharacteristics::new(b.f(8)),
                MatrixCoefficients::new(b.f(8)),
            )
        } else {
            (
                ColorPrimaries::Unspecified,
                TransferCharacteristics::Unspecified,
                MatrixCoefficients::Unspecified,
            )
        };

        let color_range: bool;
        let subsampling_x: bool;
        let subsampling_y: bool;
        let mut chroma_sample_position = ChromaSamplePosition::Unknown;

        if monochrome {
            return ColorConfig {
                separate_uv_delta_q: false,
                color_range: b.f(1) != 0,
                subsampling_x: true,
                subsampling_y: true,
                chroma_sample_position,
            };
        } else if matches!(color_primaries, ColorPrimaries::Bt709)
            && matches!(transfer_characteristics, TransferCharacteristics::Srgb)
            && matches!(matrix_coefficients, MatrixCoefficients::Identity)
        {
            color_range = true;
            subsampling_x = false;
            subsampling_y = false;
        } else {
            color_range = b.f(1) != 0;
            if seq_profile as u64 == 0 {
                subsampling_x = true;
                subsampling_y = true;
            } else if seq_profile as u64 == 1 {
                subsampling_x = false;
                subsampling_y = false;
            } else if self.bit_depth as u64 == 12 {
                subsampling_x = b.f(1) != 0;
                if subsampling_x {
                    subsampling_y = b.f(1) != 0;
                } else {
                    subsampling_y = false;
                }
            } else {
                subsampling_x = true;
                subsampling_y = false;
            }

            if subsampling_x && subsampling_y {
                chroma_sample_position = ChromaSamplePosition::new(b.f(2));
            }
        }

        ColorConfig {
            separate_uv_delta_q: b.f(1) != 0,
            color_range,
            subsampling_x,
            subsampling_y,
            chroma_sample_position,
        }
    }
}
