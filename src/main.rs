use std::{env, path::PathBuf};

use anyhow::Context;
use tracing::{error, info};

mod decoder;

fn main() {
    tracing_subscriber::fmt().init();

    let path_arg = env::args().nth(1).context("no file path provided").unwrap();
    match decoder::decode(PathBuf::from(&path_arg)) {
        Ok(_) => info!("done"),
        Err(err) => error!("{err:?}"),
    }
}
