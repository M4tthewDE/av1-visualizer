use tracing::info;

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

#[derive(Debug)]
pub enum Obu {
    TemporalDelimiter { header: ObuHeader },
    SequenceHeader { header: ObuHeader },
}

impl Obu {
    pub fn new(b: &mut BitStream) -> Obu {
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
            ObuType::SequenceHeader => Obu::sequence_header(b, header),
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

    fn sequence_header(b: &mut BitStream, header: ObuHeader) -> Obu {
        let seq_profile = b.f(3);
        let _still_picture = b.f(1) != 0;
        let reduced_still_picture_header = b.f(1) != 0;

        let decoder_model_info_present: bool;
        let mut operating_point_idc: Vec<u64> = Vec::new();
        let mut seq_level_idx: Vec<u64> = Vec::new();
        let mut seq_tier: Vec<u64> = Vec::new();
        let mut decoder_model_info_present_for_this_op: Vec<bool> = Vec::new();

        if reduced_still_picture_header {
            todo!("reduced_still_picture_header == true");
        } else {
            let timing_info_present = b.f(1) != 0;
            if timing_info_present {
                todo!("timing_info_present_flag == true");
            } else {
                decoder_model_info_present = false;
            }

            let initial_display_delay_present = b.f(1) != 0;
            let operating_points_cnt = b.f(5) + 1;

            for i in 0..operating_points_cnt as usize {
                operating_point_idc.push(b.f(12));
                seq_level_idx.push(b.f(5));

                if seq_level_idx[i] > 7 {
                    seq_tier.push(b.f(1));
                } else {
                    seq_tier.push(0);
                }

                if decoder_model_info_present {
                    todo!();
                } else {
                    decoder_model_info_present_for_this_op.push(false);
                }

                if initial_display_delay_present {
                    todo!("initial_display_delay_present == true");
                }
            }
        }

        let frame_width_bits = b.f(4) + 1;
        let frame_height_bits = b.f(4) + 1;
        let _max_frame_width = b.f(frame_width_bits) + 1;
        let _max_frame_height = b.f(frame_height_bits) + 1;

        let frame_id_numbers_present = if reduced_still_picture_header {
            false
        } else {
            b.f(1) != 0
        };

        if frame_id_numbers_present {
            todo!("frame_id_numbers_present == true");
        }

        let _use_128x128_superblock = b.f(1) != 0;
        let _enable_filter_intra = b.f(1) != 0;
        let _enable_intra_edge_filter = b.f(1) != 0;

        if reduced_still_picture_header {
            todo!();
        } else {
            let _enable_interintra_compound = b.f(1) != 0;
            let _enable_masked_compound = b.f(1) != 0;
            let _enable_warped_motion = b.f(1) != 0;
            let _enable_dual_filter = b.f(1) != 0;
            let enable_order_hint = b.f(1) != 0;

            let (_enable_jnt_comp, _enable_ref_frame_mvs) = if enable_order_hint {
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

            let _seq_force_integer_mv = if seq_force_screen_content_tools > 0 {
                if b.f(1) != 0 {
                    2
                } else {
                    b.f(1)
                }
            } else {
                2
            };

            let _order_hint_bits = if enable_order_hint { b.f(3) + 1 } else { 0 };
        }

        let _enable_superres = b.f(1) != 0;
        let _enable_cdef = b.f(1) != 0;
        let _enable_restoration = b.f(1) != 0;
        let _color_config = ColorConfig::new(b, seq_profile);
        let _film_grain_present = b.f(1) != 0;

        Obu::SequenceHeader { header }
    }
}

#[derive(Debug)]
enum ColorPrimaries {
    Unspecified,
    Bt709,
}

impl ColorPrimaries {
    fn new(val: u64) -> ColorPrimaries {
        match val {
            2 => ColorPrimaries::Unspecified,
            v => panic!("invalid value for ColorPrimaries: {val}"),
        }
    }
}

#[derive(Debug)]
enum TransferCharacteristics {
    Unspecified,
    Srgb,
}

impl TransferCharacteristics {
    fn new(val: u64) -> TransferCharacteristics {
        match val {
            2 => TransferCharacteristics::Unspecified,
            13 => TransferCharacteristics::Srgb,
            v => panic!("invalid value for TransferCharacterstics: {val}"),
        }
    }
}

#[derive(Debug)]
enum MatrixCoefficients {
    Unspecified,
    Identity,
}

impl MatrixCoefficients {
    fn new(val: u64) -> MatrixCoefficients {
        match val {
            0 => MatrixCoefficients::Identity,
            2 => MatrixCoefficients::Unspecified,
            v => panic!("invalid value for MatrixCoefficients: {val}"),
        }
    }
}

#[derive(Debug)]
pub struct ColorConfig {}

impl ColorConfig {
    fn new(b: &mut BitStream, seq_profile: u64) -> ColorConfig {
        let high_bitdepth = b.f(1) != 0;

        let bit_depth = if seq_profile == 2 && high_bitdepth {
            if b.f(1) != 0 {
                12
            } else {
                10
            }
        } else {
            if high_bitdepth {
                10
            } else {
                8
            }
        };

        let monochrome = if seq_profile == 1 { false } else { b.f(1) != 0 };
        let _num_planes = if monochrome { 1 } else { 3 };

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

        if monochrome {
            todo!("monochrome == true");
        } else if matches!(color_primaries, ColorPrimaries::Bt709)
            && matches!(transfer_characteristics, TransferCharacteristics::Srgb)
            && matches!(matrix_coefficients, MatrixCoefficients::Identity)
        {
            todo!();
        } else {
            let _color_range = b.f(1);
            let (subsampling_x, subsampling_y) = if seq_profile == 0 {
                (true, true)
            } else if seq_profile == 1 {
                (false, false)
            } else {
                if bit_depth == 12 {
                    let sub_x = b.f(1) != 0;
                    if sub_x {
                        (sub_x, b.f(1) != 0)
                    } else {
                        (sub_x, false)
                    }
                } else {
                    (true, false)
                }
            };

            if subsampling_x && subsampling_y {
                let _chroma_sample_position = b.f(2);
            }
        }

        let _separate_uv_delta_q = b.f(1) != 0;

        ColorConfig {}
    }
}
