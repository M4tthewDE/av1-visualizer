use std::io::{Cursor, Read};

use anyhow::{bail, Result};

/// Contains general presentation information, independent of the coding, for video media.
///
/// Box Type: 'vmhd'
/// Mandatory: Yes
/// Quantity: Exactly one
#[derive(Clone, Debug, Default)]
pub struct Vmhd {
    pub version: u8,
    pub flags: [u8; 3],

    /// Specifies a composition mode for this video track, from the following enumerated
    /// set, which may be extended by derived specifications:
    /// copy = 0 copy over the existing image
    pub graphics_mode: u16,

    ///  Available for use by graphics modes
    pub red: u16,

    ///  Available for use by graphics modes
    pub green: u16,

    ///  Available for use by graphics modes
    pub blue: u16,
}

impl Vmhd {
    #[tracing::instrument(skip_all, name = "vmhd")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Vmhd> {
        let mut version = [0u8; 1];
        c.read_exact(&mut version)?;

        let mut flags = [0u8; 3];
        c.read_exact(&mut flags)?;

        if flags != [0, 0, 1] {
            bail!("flags {flags:?} has to be [0, 0, 1]");
        }

        let mut graphics_mode = [0u8; 2];
        c.read_exact(&mut graphics_mode)?;
        let graphics_mode = u16::from_be_bytes(graphics_mode);

        if graphics_mode != 0 {
            bail!("graphics_mode {graphics_mode} has to be 0");
        }

        let mut red = [0u8; 2];
        c.read_exact(&mut red)?;

        let mut green = [0u8; 2];
        c.read_exact(&mut green)?;

        let mut blue = [0u8; 2];
        c.read_exact(&mut blue)?;

        Ok(Vmhd {
            version: version[0],
            flags,
            graphics_mode,
            red: u16::from_be_bytes(red),
            green: u16::from_be_bytes(green),
            blue: u16::from_be_bytes(blue),
        })
    }
}
