use tracing::{info, warn};

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
pub struct UncompressedHeader {
    pub show_existing_frame: bool,
    pub force_integer_mv: u64,
    pub current_frame_id: u64,
    pub allow_high_precision_mv: bool,
    pub disabled_frame_end_update_cdf: bool,
    pub delta_q_res: u64,
    pub delta_lf_present: bool,
    pub delta_lf_res: u64,
    pub delta_lf_multi: bool,
    pub loop_filter_params: LoopFilterParams,
    pub cdef_params: CdefParams,
    pub skip_mode_allowed: bool,
    pub skip_mode_present: bool,
    pub reduced_tx_set: bool,
}

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
            let uh = self.uncompressed_header(b);

            if uh.show_existing_frame {
                todo!();
            } else {
                self.tile_num = 0;
                self.seen_frame_header = true;
            }
        }
    }

    const NUM_REF_FRAMES: u64 = 8;
    pub const REFS_PER_FRAME: u64 = 7;
    const PRIMARY_REF_NONE: u64 = 7;

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
            error_resilient_mode = false;
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

            for i in 0..Decoder::REFS_PER_FRAME {
                self.order_hints[Decoder::LAST_FRAME + i as usize] = false;
            }
        }

        let disable_cdf_update = b.f(1) != 0;
        let allow_screen_content_tools = if self.sequence_header.seq_force_screen_content_tools
            == Decoder::SELECT_SCREEN_CONTENT_TOOLS
        {
            b.f(1)
        } else {
            self.sequence_header.seq_force_screen_content_tools
        };

        let force_integer_mv = if allow_screen_content_tools != 0 {
            if self.sequence_header.seq_force_integer_mv == Decoder::SELECT_INTEGER_MV {
                b.f(1)
            } else {
                self.sequence_header.seq_force_integer_mv
            }
        } else if self.frame_is_intra {
            1
        } else {
            0
        };

        let current_frame_id = if self.sequence_header.frame_id_numbers_present {
            todo!("frame_id_numbers_present");
        } else {
            0
        };

        let frame_size_override = if matches!(frame_type, FrameType::Switch) {
            true
        } else if self.sequence_header.reduced_still_picture_header {
            false
        } else {
            b.f(1) != 0
        };

        self.order_hint = b.f(self.order_hint_bits);

        let primary_ref_frame = if self.frame_is_intra || error_resilient_mode {
            Decoder::PRIMARY_REF_NONE
        } else {
            b.f(3)
        };

        if self.sequence_header.decoder_model_info_present {
            todo!();
        }

        let allow_high_precision_mv = false;
        let use_ref_frame_mvs = false;
        let mut allow_intrabc = false;

        let refresh_frame_flags = if matches!(frame_type, FrameType::Switch)
            || (matches!(frame_type, FrameType::Key) && show_frame)
        {
            all_frames
        } else {
            b.f(8)
        };

        if !self.frame_is_intra || refresh_frame_flags != all_frames {
            todo!();
        }

        if self.frame_is_intra {
            self.frame_size(b, frame_size_override);
            self.render_size(b);

            if allow_screen_content_tools != 0 && self.upscaled_width == self.frame_width {
                allow_intrabc = b.f(1) != 0;
            }
        } else {
            todo!();
        }

        let disabled_frame_end_update_cdf =
            if self.sequence_header.reduced_still_picture_header || disable_cdf_update {
                true
            } else {
                b.f(1) != 0
            };

        if primary_ref_frame == Decoder::PRIMARY_REF_NONE {
            warn!("init_non_coeff_cdfs() not implemented yet");
            warn!("setup_past_independence() not implemented yet");
        } else {
            todo!();
        }

        if use_ref_frame_mvs {
            todo!();
        }

        self.tile_info(b);
        let quantization_params = self.quantization_params(b);
        let segmentation_enabled = self.segmentation_params(b);

        let delta_q_present = if quantization_params.base_q_idx > 0 {
            b.f(1) != 0
        } else {
            false
        };
        let delta_q_res = if delta_q_present { b.f(2) } else { 0 };

        let mut delta_lf_present = false;
        let mut delta_lf_res = 0;
        let mut delta_lf_multi = false;
        if delta_q_present {
            if !allow_intrabc {
                delta_lf_present = b.f(1) != 0;
            }

            if delta_lf_present {
                delta_lf_res = b.f(2);
                delta_lf_multi = b.f(1) != 0
            }
        }

        if primary_ref_frame == Decoder::PRIMARY_REF_NONE {
            warn!("init_coeff_cdfs() not implemented");
        } else {
            todo!();
        }

        self.coded_lossless = true;
        self.lossless_array = vec![false; Decoder::MAX_SEGMENTS];

        for segment_id in 0..Decoder::MAX_SEGMENTS {
            let qindex = self.get_qindex(
                true,
                segment_id,
                segmentation_enabled,
                delta_q_present,
                quantization_params.base_q_idx,
            );

            self.lossless_array[segment_id] = qindex == 0
                && self.deltaq_ydc == 0
                && self.deltaq_uac == 0
                && self.deltaq_udc == 0
                && self.deltaq_vac == 0
                && self.deltaq_vdc == 0;

            if !self.lossless_array[segment_id] {
                self.coded_lossless = false;
            }

            if quantization_params.using_qmatrix {
                todo!();
            }
        }

        self.all_lossless = self.coded_lossless && (self.frame_width == self.upscaled_width);
        let loop_filter_params = self.loop_filter_params(b, allow_intrabc);
        let cdef_params = self.cdef_params(b, allow_intrabc);
        self.lr_params(b, allow_intrabc);
        self.read_tx_mode(b);
        let reference_select = if self.frame_is_intra {
            false
        } else {
            b.f(1) != 0
        };

        let (skip_mode_allowed, skip_mode_present) = self.skip_mode_params(b, reference_select);
        let allow_warped_motion = if self.frame_is_intra
            || error_resilient_mode
            || !self.sequence_header.enable_warped_motion
        {
            false
        } else {
            b.f(1) != 0
        };
        let reduced_tx_set = b.f(1) != 0;
        self.global_motion_params();
        self.film_grain_params(show_frame, showable_frame);

        UncompressedHeader {
            show_existing_frame,
            force_integer_mv,
            current_frame_id,
            allow_high_precision_mv,
            disabled_frame_end_update_cdf,
            delta_q_res,
            delta_lf_present,
            delta_lf_res,
            delta_lf_multi,
            loop_filter_params,
            cdef_params,
            skip_mode_allowed,
            skip_mode_present,
            reduced_tx_set,
        }
    }

    fn film_grain_params(&self, show_frame: bool, showable_frame: bool) {
        if !self.sequence_header.film_grain_params_present || (!show_frame && !showable_frame) {
            warn!("reset_grain_params() is not implemented");
            return;
        }

        todo!();
    }

    pub const LAST_FRAME: usize = 1;
    const ALTREF_FRAME: usize = 7;
    const WARPEDMODEL_PREC_BITS: u64 = 16;

    fn global_motion_params(&mut self) {
        let mut gm_params = vec![vec![0u64; 6]; 8];
        for r in Decoder::LAST_FRAME..=Decoder::ALTREF_FRAME {
            self.gm_type[r] = WarpModel::Identity;

            for i in 0..6 {
                gm_params[r][i] = if i % 3 == 2 {
                    1 << Decoder::WARPEDMODEL_PREC_BITS
                } else {
                    0
                };
            }
        }
    }

    fn skip_mode_params(&self, b: &mut BitStream, reference_select: bool) -> (bool, bool) {
        let skip_mode_allowed: bool;
        if self.frame_is_intra || !reference_select || !self.sequence_header.enable_order_hint {
            skip_mode_allowed = false;
        } else {
            todo!();
        }

        if skip_mode_allowed {
            (skip_mode_allowed, b.f(1) != 0)
        } else {
            (skip_mode_allowed, false)
        }
    }

    fn read_tx_mode(&mut self, b: &mut BitStream) {
        self.tx_mode = if self.coded_lossless {
            TxMode::Only4x4
        } else {
            if b.f(1) != 0 {
                TxMode::Select
            } else {
                TxMode::Largest
            }
        }
    }

    const RESTORE_NONE: u64 = 0;

    fn lr_params(&mut self, _b: &mut BitStream, allow_intrabc: bool) {
        if self.all_lossless || allow_intrabc || !self.sequence_header.enable_restoration {
            self.frame_restoration_type = vec![Decoder::RESTORE_NONE; 3];
            self.uses_lr = false;
            return;
        }
    }

    fn cdef_params(&mut self, _b: &mut BitStream, allow_intrabc: bool) -> CdefParams {
        if self.coded_lossless || allow_intrabc || self.sequence_header.enable_cdef {
            self.cdef_damping = 3;
            return CdefParams {
                cdef_bits: 0,
                cdef_y_pri_strength: vec![0; 1],
                cdef_y_sec_strength: vec![0; 1],
                cdef_uv_pri_strength: vec![0; 1],
                cdef_uv_sec_strength: vec![0; 1],
            };
        }

        todo!();
    }

    fn loop_filter_params(&self, b: &mut BitStream, allow_intrabc: bool) -> LoopFilterParams {
        let mut loop_filter_level = [0u64; 4];

        if self.coded_lossless || allow_intrabc {
            todo!();
        }

        loop_filter_level[0] = b.f(6);
        loop_filter_level[1] = b.f(6);

        if matches!(self.num_planes, NumPlanes::Three) {
            if loop_filter_level[0] != 0 || loop_filter_level[1] != 0 {
                loop_filter_level[2] = b.f(6);
                loop_filter_level[3] = b.f(6);
            }
        }

        let loop_filter_sharpness = b.f(3);
        let loop_filter_delta_enabled = b.f(1) != 0;

        if loop_filter_delta_enabled {
            todo!();
        }

        LoopFilterParams {
            loop_filter_level,
            loop_filter_sharpness,
            loop_filter_delta_enabled,
        }
    }

    fn get_qindex(
        &self,
        ignore_delta_q: bool,
        segment_id: usize,
        segmentation_enabled: bool,
        delta_q_present: bool,
        base_q_idx: u64,
    ) -> u64 {
        if segmentation_enabled && self.feature_enabled[segment_id][Decoder::SEG_LVL_ALT_Q] {
            todo!();
        } else if !ignore_delta_q && delta_q_present {
            self.current_q_index
        } else {
            base_q_idx
        }
    }

    const MAX_SEGMENTS: usize = 8;
    const SEG_LVL_MAX: usize = 8;
    const SEG_LVL_REF_FRAME: usize = 5;
    const SEG_LVL_ALT_Q: usize = 0;

    fn segmentation_params(&mut self, b: &mut BitStream) -> bool {
        let segmentation_enabled = b.f(1) != 0;
        if segmentation_enabled {
            todo!();
        } else {
            self.feature_enabled = vec![vec![false; Decoder::SEG_LVL_MAX]; Decoder::MAX_SEGMENTS];
            self.feature_data = vec![vec![0; Decoder::SEG_LVL_MAX]; Decoder::MAX_SEGMENTS];

            for i in 0..Decoder::MAX_SEGMENTS {
                for j in 0..Decoder::SEG_LVL_MAX {
                    self.feature_enabled[i][j] = false;
                    self.feature_data[i][j] = 0;
                }
            }
        }

        self.seg_id_pre_skip = false;
        self.last_active_seg_id = 0;
        for i in 0..Decoder::MAX_SEGMENTS {
            for j in 0..Decoder::SEG_LVL_MAX {
                if self.feature_enabled[i][j] {
                    self.last_active_seg_id = i as u64;
                    if j >= Decoder::SEG_LVL_REF_FRAME {
                        self.seg_id_pre_skip = true;
                    }
                }
            }
        }

        segmentation_enabled
    }

    fn quantization_params(&mut self, b: &mut BitStream) -> QuantizationParams {
        let base_q_idx = b.f(8);
        self.deltaq_ydc = Decoder::read_delta_q(b);

        if matches!(self.num_planes, NumPlanes::Three) {
            let diff_uv_delta = if self.sequence_header.color_config.separate_uv_delta_q {
                b.f(1) != 0
            } else {
                false
            };

            self.deltaq_udc = Decoder::read_delta_q(b);
            self.deltaq_uac = Decoder::read_delta_q(b);

            if diff_uv_delta {
                self.deltaq_vdc = Decoder::read_delta_q(b);
                self.deltaq_vac = Decoder::read_delta_q(b);
                self.deltaq_vdc = self.deltaq_udc;
                self.deltaq_vac = self.deltaq_uac;
            }
        } else {
            self.deltaq_udc = 0;
            self.deltaq_uac = 0;
            self.deltaq_vdc = 0;
            self.deltaq_vac = 0;
        }

        let using_qmatrix = b.f(1) != 0;
        if using_qmatrix {
            let qm_y = b.f(4);
            let qm_u = b.f(4);
            let qm_v = if !self.sequence_header.color_config.separate_uv_delta_q {
                qm_u
            } else {
                b.f(4)
            };

            QuantizationParams {
                base_q_idx,
                qm_y,
                qm_u,
                qm_v,
                using_qmatrix,
            }
        } else {
            QuantizationParams {
                base_q_idx: 0,
                qm_y: 0,
                qm_u: 0,
                qm_v: 0,
                using_qmatrix,
            }
        }
    }

    fn read_delta_q(b: &mut BitStream) -> i64 {
        if b.f(1) != 0 {
            b.su(7)
        } else {
            0
        }
    }

    fn frame_size(&mut self, b: &mut BitStream, frame_size_override: bool) {
        if frame_size_override {
            todo!();
        } else {
            self.frame_width = self.sequence_header.max_frame_width;
            self.frame_height = self.sequence_header.max_frame_height;
        }

        self.superres_params(b);
        self.compute_image_size();
    }

    const SUPERRES_DENOM_BITS: u64 = 3;
    const SUPERRES_DENOM_MIN: u64 = 9;
    const SUPERRES_NUM: u64 = 8;

    fn superres_params(&mut self, b: &mut BitStream) {
        let use_superres = if self.sequence_header.enable_superres {
            b.f(1) != 0
        } else {
            false
        };

        self.superres_denom = if use_superres {
            b.f(Decoder::SUPERRES_DENOM_BITS) + Decoder::SUPERRES_DENOM_MIN
        } else {
            Decoder::SUPERRES_NUM
        };

        self.upscaled_width = self.frame_width;
        self.frame_width = (self.upscaled_width * Decoder::SUPERRES_NUM
            + (self.superres_denom / 2))
            / self.superres_denom;
    }

    fn compute_image_size(&mut self) {
        self.mi_cols = 2 * ((self.frame_width + 7) >> 3);
        self.mi_rows = 2 * ((self.frame_height + 7) >> 3);
    }

    fn render_size(&mut self, b: &mut BitStream) {
        if b.f(1) != 0 {
            self.render_width = b.f(16) + 1;
            self.render_height = b.f(16) + 1;
        } else {
            self.render_width = self.upscaled_width;
            self.render_height = self.frame_height;
        }
    }

    const MAX_TILE_WIDTH: u64 = 4096;
    const MAX_TILE_AREA: u64 = 4096 * 2304;
    const MAX_TILE_COLS: u64 = 64;
    const MAX_TILE_ROWS: u64 = 64;

    fn tile_info(&mut self, b: &mut BitStream) {
        let (sb_cols, sb_rows, sb_shift) = if self.sequence_header.use_128x128_superblock {
            (((self.mi_cols + 31) >> 5), (self.mi_rows + 31) >> 5, 5)
        } else {
            (((self.mi_cols + 15) >> 4), (self.mi_rows + 15) >> 4, 4)
        };

        let sb_size = sb_shift + 2;
        let max_tile_width_sb = Decoder::MAX_TILE_WIDTH >> sb_size;
        let max_tile_area_sb = Decoder::MAX_TILE_AREA >> (2 * sb_size);
        let min_log2_tile_cols = Decoder::tile_log2(max_tile_width_sb, sb_cols);
        let max_log2_tile_cols = Decoder::tile_log2(1, sb_cols.min(Decoder::MAX_TILE_COLS));
        let max_log2_tile_rows = Decoder::tile_log2(1, sb_rows.min(Decoder::MAX_TILE_ROWS));
        let min_log2_tiles =
            min_log2_tile_cols.max(Decoder::tile_log2(max_tile_area_sb, sb_rows * sb_cols));

        let uniform_tile_spacing = b.f(1) != 0;
        if uniform_tile_spacing {
            self.tile_cols_log2 = min_log2_tile_cols;

            while self.tile_cols_log2 >= max_log2_tile_cols {
                if b.f(1) != 0 {
                    self.tile_cols_log2 += 1;
                } else {
                    break;
                }
            }

            let tile_width_sb = (sb_cols + (1 << self.tile_cols_log2) - 1) >> self.tile_cols_log2;
            let mut i = 0;
            self.mi_col_starts = vec![0; sb_cols as usize];
            for start_sb in (0..sb_cols).step_by(tile_width_sb as usize) {
                self.mi_col_starts[i] = start_sb << sb_shift;
                i += 1;
            }
            self.mi_col_starts[i] = self.mi_cols;
            self.tile_cols = i as u64;

            self.tile_rows_log2 = 0.max(min_log2_tiles - self.tile_cols_log2);
            while self.tile_rows_log2 < max_log2_tile_rows {
                if b.f(1) != 0 {
                    self.tile_rows_log2 += 1;
                } else {
                    break;
                }
            }

            let tile_height_sb = (sb_rows + (1 << self.tile_rows_log2) - 1) >> self.tile_rows_log2;
            let mut i = 0;
            self.mi_row_starts = vec![0; sb_rows as usize];
            for start_sb in (0..sb_rows).step_by(tile_height_sb as usize) {
                self.mi_row_starts[i] = start_sb << sb_shift;
                i += 1;
            }

            self.mi_row_starts[i] = self.mi_rows;
            self.tile_rows = i as u64;
        } else {
            todo!("no uniform tile spacing");
        }

        if self.tile_cols_log2 > 0 || self.tile_rows_log2 > 0 {
            let _context_update_tile_id = b.f(self.tile_rows_log2 + self.tile_cols_log2);
            self.tile_size_bytes = b.f(2) + 1;
        } else {
            let _context_update_tile_id = 0;
        }
    }

    fn tile_log2(blk_size: u64, target: u64) -> u64 {
        let mut k = 0;
        loop {
            if (blk_size << k) >= target {
                break;
            }

            k += 1;
        }

        k
    }
}

#[derive(Debug)]
struct QuantizationParams {
    pub base_q_idx: u64,
    pub qm_y: u64,
    pub qm_u: u64,
    pub qm_v: u64,
    pub using_qmatrix: bool,
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

#[derive(Debug, Default)]
pub struct LoopFilterParams {
    pub loop_filter_level: [u64; 4],
    pub loop_filter_sharpness: u64,
    pub loop_filter_delta_enabled: bool,
}

#[derive(Debug, Default)]
pub struct CdefParams {
    pub cdef_bits: u64,
    pub cdef_y_pri_strength: Vec<u64>,
    pub cdef_y_sec_strength: Vec<u64>,
    pub cdef_uv_pri_strength: Vec<u64>,
    pub cdef_uv_sec_strength: Vec<u64>,
}

#[derive(Debug)]
pub enum TxMode {
    Invalid = -1,
    Only4x4 = 0,
    Largest = 1,
    Select = 2,
}

impl Default for TxMode {
    fn default() -> Self {
        Self::Invalid
    }
}

#[derive(Debug)]
pub enum WarpModel {
    Invalid = -1,
    Identity = 0,
    Translation = 1,
    Rotzoom = 2,
    Affine = 3,
}

impl Default for WarpModel {
    fn default() -> Self {
        Self::Invalid
    }
}
