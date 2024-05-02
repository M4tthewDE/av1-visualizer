use std::io::{Cursor, Read};

use anyhow::{bail, Result};

/// https://aomediacodec.github.io/av1-isobmff/#av1sampleentry-section
#[derive(Clone, Debug, Default)]
pub struct Av1C {
    pub marker: u8,
    pub version: u8,
    pub seq_profile: u8,
    pub seq_level_idx_0: u8,
    pub seq_tier_0: bool,
    pub high_bitdepth: bool,
    pub twelve_bit: bool,
    pub monochrome: bool,
    pub chroma_subsampling_x: bool,
    pub chroma_subsampling_y: bool,
    pub chroma_sample_position: u8,
    pub initial_presentation_delay_minus_one: Option<u8>,
    pub config_obus: Vec<u8>,
}

impl Av1C {
    #[tracing::instrument(skip_all, name = "av1C")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: u64) -> Result<Av1C> {
        let mut marker_and_version = [0u8; 1];
        c.read_exact(&mut marker_and_version)?;

        let marker = marker_and_version[0] >> 7;
        if marker != 1 {
            bail!("marker {marker} has to be 1");
        }

        let version = marker_and_version[0] & 127;
        if version != 1 {
            bail!("version {version} is not supported");
        }

        let mut seq_profile_and_seq_level_idx_0 = [0u8; 1];
        c.read_exact(&mut seq_profile_and_seq_level_idx_0)?;

        let seq_profile = seq_profile_and_seq_level_idx_0[0] >> 5;

        let seq_level_idx_0 = seq_profile_and_seq_level_idx_0[0] & 31;

        let mut params = [0u8; 1];
        c.read_exact(&mut params)?;

        let mut delay = [0u8; 1];
        c.read_exact(&mut delay)?;

        let initial_presentation_delay_present = delay[0] >> 7;
        let initial_presentation_delay_minus_one = if initial_presentation_delay_present == 1 {
            Some(delay[0] & 16)
        } else {
            None
        };

        let mut config_obus = Vec::new();
        // unsure why we need to subtract 12 here
        for _ in 0..size - 12 {
            let mut co = [0u8; 1];
            c.read_exact(&mut co)?;
            config_obus.push(co[0]);
        }

        Ok(Av1C {
            marker,
            version,
            seq_profile,
            seq_level_idx_0,
            seq_tier_0: (params[0] >> 7) != 0,
            high_bitdepth: ((params[0] >> 6) & 1) != 0,
            twelve_bit: ((params[0] >> 5) & 1) != 0,
            monochrome: ((params[0] >> 4) & 1) != 0,
            chroma_subsampling_x: ((params[0] >> 3) & 1) != 0,
            chroma_subsampling_y: ((params[0] >> 2) & 1) != 0,
            chroma_sample_position: params[0] & 2,
            initial_presentation_delay_minus_one,
            config_obus,
        })
    }
}
