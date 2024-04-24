use anyhow::Result;
use std::io::{Cursor, Read};
use tracing::info;

#[derive(Clone, Debug, Default)]
pub struct Ftyp {
    pub major_brand: String,
    pub minor_version: u32,
    pub brands: Vec<String>,
}

#[tracing::instrument(skip_all)]
pub fn ftyp(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<Ftyp> {
    let mut major_brand = [0u8; 4];
    c.read_exact(&mut major_brand)?;
    let major_brand = String::from_utf8(major_brand.to_vec())?;

    let mut minor_version = [0u8; 4];
    c.read_exact(&mut minor_version)?;
    let minor_version = u32::from_be_bytes(minor_version);
    info!("major_brand: {major_brand}, minor_version: {minor_version}",);

    let mut brands = Vec::new();
    for _ in 0..size / 8 {
        let mut brand = [0u8; 4];
        c.read_exact(&mut brand)?;
        brands.push(String::from_utf8(brand.to_vec())?);
    }

    info!("brands: {brands:?}");

    Ok(Ftyp {
        major_brand,
        minor_version,
        brands,
    })
}
