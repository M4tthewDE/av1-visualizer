use std::{env, path::PathBuf};

use anyhow::{Context, Result};

mod decoder;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let path_arg = env::args().nth(1).context("no file path provided")?;
    decoder::decode(PathBuf::from(&path_arg))?;

    Ok(())
}
