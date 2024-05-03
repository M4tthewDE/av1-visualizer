use std::io::{Cursor, Read};

use anyhow::Result;

use crate::decoder::mp4::{fixed16, fixed32};

/// Defines overall information which is media‐independent, and relevant to the entire
/// presentation considered as a whole.
///
/// Box Type: 'mvhd'
/// Mandatory: Yes
/// Quantity: Exactly one
#[derive(Clone, Debug, Default)]
pub struct Mvhd {
    pub version: u8,

    /// Creation time of the presentation (in seconds since midnight, Jan. 1, 1904, in UTC time)
    pub creation_time: u32,

    /// Most recent time the presentation was modified (in seconds since midnight, Jan. 1, 1904, in UTC time)
    pub modification_time: u32,

    /// Time-scale for the entire presentation; this is the number of time units that pass in one second.
    /// For example, a time coordinate system that measures time in sixtieths of a second has a time scale of 60.
    pub timescale: u32,

    /// Length of the presentation (in the indicated timescale). This
    /// property is derived from the presentation’s tracks: the value of this field corresponds to the
    /// duration of the longest track in the presentation. If the duration cannot be determined then
    /// duration is set to all 1s.
    pub duration: u32,

    /// Indicates the preferred rate to play the presentation; 1.0 (0x00010000) is normal forward playback.
    pub rate: f64,

    /// Preferred playback volume, 1.0 (0x0100) is full volume.
    pub volume: f64,

    /// Transformation matrix for the video; (u,v,w) are restricted here to (0,0,1), hex values (0,0,0x40000000).
    pub matrix: [u32; 9],

    pub pre_defined: [u32; 6],

    /// Value to use for the track ID of the next track
    /// to be added to this presentation. Zero is not a valid track ID value. The value of
    /// next_track_id shall be larger than the largest track‐ID in use. If this value is equal to all 1s
    /// (32-bit maxint), and a new media track is to be added, then a search must be made in the file for
    /// an unused track identifier.
    pub next_track_id: u32,
}

impl Mvhd {
    #[tracing::instrument(skip_all, name = "mvhd")]
    pub fn new(c: &mut Cursor<Vec<u8>>) -> Result<Mvhd> {
        let mut version = [0u8; 4];
        c.read_exact(&mut version)?;

        let mut creation_time = [0u8; 4];
        c.read_exact(&mut creation_time)?;

        let mut modification_time = [0u8; 4];
        c.read_exact(&mut modification_time)?;

        let mut timescale = [0u8; 4];
        c.read_exact(&mut timescale)?;

        let mut duration = [0u8; 4];
        c.read_exact(&mut duration)?;

        let mut rate = [0u8; 4];
        c.read_exact(&mut rate)?;

        let mut volume = [0u8; 2];
        c.read_exact(&mut volume)?;

        // skip 10 reserved bytes
        c.set_position(c.position() + 10);

        let mut matrix = [0_u32; 9];
        for m in &mut matrix {
            let mut val = [0u8; 4];
            c.read_exact(&mut val)?;
            *m = u32::from_be_bytes(val);
        }

        let mut pre_defined = [0_u32; 6];
        for p in &mut pre_defined {
            let mut val = [0u8; 4];
            c.read_exact(&mut val)?;
            *p = u32::from_be_bytes(val);
        }

        let mut next_track_id = [0u8; 4];
        c.read_exact(&mut next_track_id)?;

        Ok(Mvhd {
            version: version[0],
            creation_time: u32::from_be_bytes(creation_time),
            modification_time: u32::from_be_bytes(modification_time),
            timescale: u32::from_be_bytes(timescale),
            duration: u32::from_be_bytes(duration),
            rate: fixed32(rate),
            volume: fixed16(volume),
            matrix,
            pre_defined,
            next_track_id: u32::from_be_bytes(next_track_id),
        })
    }
}
