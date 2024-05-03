use anyhow::Result;
use std::io::{Cursor, Read};

/// Identifies the specifications to which this file complies.
///
/// Box Type: 'ftyp'
/// Mandatory: Yes
/// Quantity: Exactly one
#[derive(Clone, Debug, Default)]
pub struct Ftyp {
    /// Printable four-character code, registered with ISO, that identifies a precise specification
    pub major_brand: String,

    /// Informative integer for the minor version of the major brand
    pub minor_version: u32,

    /// Compatible brand specifications
    pub compatible_brands: Vec<String>,
}

impl Ftyp {
    #[tracing::instrument(skip_all, name = "ftyp")]
    pub fn new(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Ftyp> {
        let mut major_brand = [0u8; 4];
        c.read_exact(&mut major_brand)?;
        let major_brand = String::from_utf8(major_brand.to_vec())?;

        let mut minor_version = [0u8; 4];
        c.read_exact(&mut minor_version)?;
        let minor_version = u32::from_be_bytes(minor_version);

        let mut compatible_brands = Vec::new();
        for _ in 0..size / 8 {
            let mut brand = [0u8; 4];
            c.read_exact(&mut brand)?;
            compatible_brands.push(String::from_utf8(brand.to_vec())?);
        }

        Ok(Ftyp {
            major_brand,
            minor_version,
            compatible_brands,
        })
    }
}
