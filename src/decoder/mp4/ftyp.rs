use anyhow::Result;
use std::io::{Cursor, Read};
use tracing::info;

/// Allows the reader to determine whether this is a type of file that the reader understands.
/// See the [Apple documentation](https://developer.apple.com/documentation/quicktime-file-format/file_type_compatibility_atom) for more information.
#[derive(Clone, Debug, Default)]
pub struct Ftyp {
    /// represents the file format code
    pub major_brand: String,
    /// indicates the file format specification version
    pub minor_version: u32,
    /// compatible file formats
    pub compatible_brands: Vec<String>,
}

#[tracing::instrument(skip_all, name = "ftyp")]
pub fn ftyp(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Ftyp> {
    let mut major_brand = [0u8; 4];
    c.read_exact(&mut major_brand)?;
    let major_brand = String::from_utf8(major_brand.to_vec())?;

    let mut minor_version = [0u8; 4];
    c.read_exact(&mut minor_version)?;
    let minor_version = u32::from_be_bytes(minor_version);
    info!("major_brand: {major_brand}, minor_version: {minor_version}",);

    let mut compatible_brands = Vec::new();
    for _ in 0..size / 8 {
        let mut brand = [0u8; 4];
        c.read_exact(&mut brand)?;
        compatible_brands.push(String::from_utf8(brand.to_vec())?);
    }

    info!("brands: {compatible_brands:?}");

    Ok(Ftyp {
        major_brand,
        minor_version,
        compatible_brands,
    })
}
