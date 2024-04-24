use std::{
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::{bail, Result};
use tracing::info;

// https://github.com/alfg/mp4/blob/master/atom/box.go#L99
#[tracing::instrument(skip_all)]
pub fn extract(p: PathBuf) -> Result<()> {
    let data = std::fs::read(p)?;
    info!("loaded {} bytes", data.len());

    let mut cursor = Cursor::new(data);

    loop {
        match read_box(&mut cursor) {
            Ok(_) => {}
            Err(err) => {
                info!("finished reading: {err:?}");
                return Ok(());
            }
        }
    }
}

#[tracing::instrument(skip_all)]
fn read_box(c: &mut Cursor<Vec<u8>>) -> Result<()> {
    let mut box_size = [0u8; 4];
    c.read_exact(&mut box_size)?;
    let box_size = u32::from_be_bytes(box_size);

    let mut box_type = [0u8; 4];
    c.read_exact(&mut box_type)?;
    let box_type = String::from_utf8(box_type.to_vec())?;

    match box_type.as_str() {
        "ftyp" => ftyp(c, box_size as usize),
        typ => bail!("TODO: {typ:?}"),
    }
}

#[tracing::instrument(skip_all)]
fn ftyp(c: &mut Cursor<Vec<u8>>, size: usize) -> Result<()> {
    let mut major_brand = [0u8; 4];
    c.read_exact(&mut major_brand)?;
    let major_brand = String::from_utf8(major_brand.to_vec())?;

    let mut minor_version = [0u8; 4];
    c.read_exact(&mut minor_version)?;
    let minor_version = u32::from_be_bytes(minor_version);
    info!("major_brand: {major_brand}, minor_version: {minor_version}",);

    let mut brands = Vec::new();
    for _ in 0..(size - 8) / 8 {
        let mut brand = [0u8; 4];
        c.read_exact(&mut brand)?;
        brands.push(String::from_utf8(brand.to_vec())?);
    }

    info!("brands: {brands:?}");

    Ok(())
}
