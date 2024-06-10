use tracing::info;

use super::{BitDepth, BitStream, Decoder, NumPlanes};

#[derive(Debug, Clone)]
pub enum ObuType {
    Reserved,
    SequenceHeader,
    TemporalDelimiter,
    TileGroup,
    TileList,
    Frame,
}

impl Default for ObuType {
    fn default() -> Self {
        Self::Reserved
    }
}

impl ObuType {
    fn new(val: u64) -> ObuType {
        match val {
            0 => ObuType::Reserved,
            1 => ObuType::SequenceHeader,
            2 => ObuType::TemporalDelimiter,
            4 => ObuType::TileGroup,
            6 => ObuType::Frame,
            8 => ObuType::TileList,
            v => panic!("unknown obu type: {v}"),
        }
    }
}

#[derive(Debug, Default)]
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
pub struct TemporalDelimiter {
    pub header: ObuHeader,
}

#[derive(Debug, Default)]
pub struct SequenceHeader {
    pub header: ObuHeader,
    pub still_picture: bool,
    pub timing_info_present: bool,
    pub decoder_model_info_present: bool,
    pub initial_display_delay_present: bool,
    pub operating_points_cnt: u64,
    pub operating_point_idc: Vec<u64>,
    pub seq_level_idx: Vec<u64>,
    pub seq_tier: Vec<u64>,
    pub decoder_model_present_for_this_op: Vec<bool>,
    pub initial_display_delay_present_for_this_op: Vec<bool>,
    pub initial_display_delay: Vec<u64>,
    pub max_frame_width: u64,
    pub max_frame_height: u64,
    pub frame_id_numbers_present: bool,
    pub use_128x128_superblock: bool,
    pub enable_filter_intra: bool,
    pub enable_intra_edge_filter: bool,
    pub enable_interintra_compound: bool,
    pub enable_masked_compound: bool,
    pub enable_warped_motion: bool,
    pub enable_dual_filter: bool,
    pub enable_order_hint: bool,
    pub enable_jnt_comp: bool,
    pub enable_ref_frame_mvs: bool,
    pub seq_force_integer_mv: u64,
    pub seq_force_screen_content_tools: u64,
    pub enable_superres: bool,
    pub enable_cdef: bool,
    pub enable_restoration: bool,
    pub color_config: ColorConfig,
    pub film_grain_params_present: bool,
    pub reduced_still_picture_header: bool,
}

#[derive(Debug, Default)]
pub struct UncompressedHeader {}

impl Decoder {
    pub fn obu(&mut self, b: &mut BitStream) {
        let header = ObuHeader::new(b);
        let size = if header.has_size {
            b.leb128()
        } else {
            todo!("where does sz come from?");
        };

        info!("size: {size}");

        let start_position = b.pos;

        let obu_type = header.obu_type.clone();

        match obu_type {
            ObuType::SequenceHeader => {
                let sh = self.sequence_header(b, header);
                info!("{sh:?}");
                self.sequence_header = sh;
            }
            ObuType::TemporalDelimiter => b.seen_frame_header = false,
            ObuType::Frame => self.frame(b),
            _ => panic!("obu type not implemented: {obu_type:?}"),
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
    }

    const SELECT_INTEGER_MV: u64 = 2;
    const SELECT_SCREEN_CONTENT_TOOLS: u64 = 2;

    fn sequence_header(&mut self, b: &mut BitStream, header: ObuHeader) -> SequenceHeader {
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
        let seq_force_screen_content_tools: u64;
        let seq_force_integer_mv: u64;

        if reduced_still_picture_header {
            enable_interintra_compound = false;
            enable_masked_compound = false;
            enable_warped_motion = false;
            enable_dual_filter = false;
            enable_order_hint = false;
            enable_jnt_comp = false;
            enable_ref_frame_mvs = false;
            seq_force_screen_content_tools = Decoder::SELECT_SCREEN_CONTENT_TOOLS;
            seq_force_integer_mv = Decoder::SELECT_INTEGER_MV;
            self.order_hint_bits = 0;
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
            seq_force_screen_content_tools = if seq_choose_screen_content_tools {
                Decoder::SELECT_SCREEN_CONTENT_TOOLS
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

        SequenceHeader {
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
            frame_id_numbers_present,
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
            seq_force_screen_content_tools,
            enable_superres: b.f(1) != 0,
            enable_cdef: b.f(1) != 0,
            enable_restoration: b.f(1) != 0,
            color_config: self.color_config(b, seq_profile),
            film_grain_params_present: b.f(1) != 0,
            reduced_still_picture_header,
        }
    }

    fn frame(&mut self, b: &mut BitStream) {
        let _start = b.pos;
        self.frame_header(b);
        todo!("after frame header parsing");
    }

    fn frame_header(&mut self, b: &mut BitStream) {
        if self.seen_frame_header {
            todo!("seen_frame_header == true");
        } else {
            self.seen_frame_header = true;
            let _uh = self.uncompressed_header(b);
            todo!("after uncompressed header parsing");
        }
    }

    const NUM_REF_FRAMES: u64 = 8;
    const REFS_PER_FRAME: u64 = 7;

    fn uncompressed_header(&mut self, b: &mut BitStream) -> UncompressedHeader {
        if self.sequence_header.frame_id_numbers_present {
            todo!("frame_id_numbers_present == true");
        }

        let all_frames = (1 << Decoder::NUM_REF_FRAMES) - 1;

        let show_existing_frame: bool;
        let frame_type: FrameType;
        let show_frame: bool;
        let showable_frame: bool;
        let error_resilient_mode: bool;

        if self.sequence_header.reduced_still_picture_header {
            show_existing_frame = false;
            frame_type = FrameType::Key;
            self.frame_is_intra = true;
            show_frame = true;
            showable_frame = false;
        } else {
            show_existing_frame = b.f(1) != 0;
            if show_existing_frame {
                todo!("show_existing_frame == true");
            }

            frame_type = FrameType::new(b.f(2));
            self.frame_is_intra =
                matches!(frame_type, FrameType::IntraOnly) || matches!(frame_type, FrameType::Key);

            show_frame = b.f(1) != 0;
            if show_frame
                && self.sequence_header.decoder_model_info_present
                && todo!("decoder model info has to be parsed at this point")
            {
                todo!("temporal_point_info()");
            }

            showable_frame = if show_frame {
                !matches!(frame_type, FrameType::Key)
            } else {
                b.f(1) != 0
            };

            error_resilient_mode = if matches!(frame_type, FrameType::Switch)
                || (matches!(frame_type, FrameType::Key) && show_frame)
            {
                true
            } else {
                b.f(1) != 0
            };
        }

        if matches!(frame_type, FrameType::Key) && show_frame {
            for i in 0..Decoder::NUM_REF_FRAMES {
                self.ref_valid[i as usize] = false;
                self.ref_order_hint[i as usize] = false;
            }

            for i in 0..Decoder::REFS_PER_FRAME {}
        }

        let disable_cdf_update = b.f(1) != 0;
        let allow_screen_content_tools = if self.sequence_header.seq_force_screen_content_tools
            == Decoder::SELECT_SCREEN_CONTENT_TOOLS
        {
            b.f(1)
        } else {
            self.sequence_header.seq_force_screen_content_tools
        };

        if allow_screen_content_tools != 0 {
            todo!("allow_screen_content_tools != 0");
        }

        todo!("uncompressed_header");
    }
}

#[derive(Debug)]
enum FrameType {
    Key = 0,
    Inter = 1,
    IntraOnly = 2,
    Switch = 3,
}

impl FrameType {
    fn new(val: u64) -> FrameType {
        match val {
            0 => FrameType::Key,
            1 => FrameType::Inter,
            2 => FrameType::IntraOnly,
            3 => FrameType::Switch,
            _ => panic!("invalid value for FrameType: {val}"),
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

impl Default for ChromaSamplePosition {
    fn default() -> Self {
        Self::Unknown
    }
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

#[derive(Debug, Default)]
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
