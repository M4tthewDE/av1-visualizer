use std::{env, path::Path};

use anyhow::{Context, Result};

mod decoder;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let path_arg = env::args().nth(1).context("no file path provided")?;
    decoder::decode(&Path::new(&path_arg));

    Ok(())
}
