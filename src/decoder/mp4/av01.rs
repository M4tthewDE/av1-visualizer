use anyhow::{bail, Result};
use std::io::{Cursor, Read};
use tracing::info;

use super::av1c::Av1C;

// https://github.com/abema/go-mp4/blob/ffc2144971771a2b983cf73eab568eaae5b9c195/box_types_iso14496_12.go#L477
#[derive(Clone, Debug, Default)]
pub struct Fiel {
    pub field_count: u8,
    pub field_ordering: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Pasp {
    pub h_spacing: u32,
    pub v_spacing: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Btrt {
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Av01 {
    pub width: u16,
    pub height: u16,
    pub horizresolution: u32,
    pub vertresolution: u32,
    pub frame_count: u16,
    pub compressor_name: String,
    pub depth: u16,
    pub av1c: Av1C,
    pub fiel: Fiel,
    // FIXME: this should be optional
    pub pasp: Pasp,
    pub btrt: Btrt,
}

impl Av01 {
    #[tracing::instrument(skip_all, name = "av01")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Av01> {
        // skip pre_defined and reserved
        c.set_position(c.position() + 16);

        let mut width = [0u8; 2];
        c.read_exact(&mut width)?;
        let width = u16::from_be_bytes(width);

        let mut height = [0u8; 2];
        c.read_exact(&mut height)?;
        let height = u16::from_be_bytes(height);

        let mut horizresolution = [0u8; 4];
        c.read_exact(&mut horizresolution)?;
        let horizresolution = u32::from_be_bytes(horizresolution);

        let mut vertresolution = [0u8; 4];
        c.read_exact(&mut vertresolution)?;
        let vertresolution = u32::from_be_bytes(vertresolution);

        // skip reserved
        c.set_position(c.position() + 4);

        let mut frame_count = [0u8; 2];
        c.read_exact(&mut frame_count)?;
        let frame_count = u16::from_be_bytes(frame_count);

        let mut compressor_name = [0u8; 4];
        c.read_exact(&mut compressor_name)?;
        let compressor_name = String::from_utf8(compressor_name.to_vec())?;

        // unsure why we need to skip 28 bytes here
        c.set_position(c.position() + 28);

        let mut depth = [0u8; 2];
        c.read_exact(&mut depth)?;
        let depth = u16::from_be_bytes(depth);

        // skip pre_defined
        c.set_position(c.position() + 2);

        let mut size = [0u8; 4];
        c.read_exact(&mut size)?;
        let size = u32::from_be_bytes(size);

        let mut config_box = [0u8; 4];
        c.read_exact(&mut config_box)?;
        let config_box = String::from_utf8(config_box.to_vec())?;

        if config_box != "av1C" {
            bail!("config box {config_box} is not supported");
        }

        let av1c = Av1C::new(c, size as u64)?;

        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;
        if box_type != "fiel" {
            bail!("only supports 'fiel', not '{box_type}'");
        }

        let mut fields = [0u8; 2];
        c.read_exact(&mut fields)?;

        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;
        if box_type != "pasp" {
            bail!("only supports 'pasp', not '{box_type}'");
        }

        let mut h_spacing = [0u8; 4];
        c.read_exact(&mut h_spacing)?;
        let h_spacing = u32::from_be_bytes(h_spacing);

        let mut v_spacing = [0u8; 4];
        c.read_exact(&mut v_spacing)?;
        let v_spacing = u32::from_be_bytes(v_spacing);

        let mut box_size = [0u8; 4];
        c.read_exact(&mut box_size)?;

        let mut box_type = [0u8; 4];
        c.read_exact(&mut box_type)?;
        let box_type = String::from_utf8(box_type.to_vec())?;

        if box_type != "btrt" {
            bail!("only supports 'btrt', not '{box_type}'");
        }

        let mut buffer_size_db = [0u8; 4];
        c.read_exact(&mut buffer_size_db)?;
        let buffer_size_db = u32::from_be_bytes(buffer_size_db);

        let mut max_bitrate = [0u8; 4];
        c.read_exact(&mut max_bitrate)?;
        let max_bitrate = u32::from_be_bytes(max_bitrate);

        let mut avg_bitrate = [0u8; 4];
        c.read_exact(&mut avg_bitrate)?;
        let avg_bitrate = u32::from_be_bytes(avg_bitrate);

        let av01 = Av01 {
            width,
            height,
            horizresolution,
            vertresolution,
            frame_count,
            compressor_name,
            depth,
            av1c,
            fiel: Fiel {
                field_count: fields[0],
                field_ordering: fields[1],
            },
            pasp: Pasp {
                h_spacing,
                v_spacing,
            },
            btrt: Btrt {
                buffer_size_db,
                max_bitrate,
                avg_bitrate,
            },
        };

        info!("av01: {av01:?}");

        Ok(av01)
    }
}
