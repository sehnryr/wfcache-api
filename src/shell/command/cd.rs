use std::path::PathBuf;

use anyhow::{Error, Result};
use clap::Parser;

use crate::shell::{error::PathNotFound, State};
use crate::utils::path::normalize_path;

/// Change the working directory
#[derive(Parser, Debug, Clone)]
pub struct Arguments {
    #[arg(default_value = "/")]
    directory: PathBuf,
}

pub fn command(state: &mut State, args: Arguments) -> Result<()> {
    let directory = normalize_path(&args.directory, &state.current_lotus_dir);

    // Get the directory node
    let dir_node = state.h_cache.get_directory_node(directory.to_str().unwrap());

    // Check if the directory exists
    if dir_node.is_none() {
        return Err(Error::from(PathNotFound));
    }

    // Set the current directory
    state.current_lotus_dir = directory;

    Ok(())
}
