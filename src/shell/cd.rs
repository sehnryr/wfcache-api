use super::{error::PathNotFound, State};
use crate::utils::path::normalize_path;
use clap::Parser;
use std::path::PathBuf;

/// Change the working directory
#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(default_value = "/")]
    directory: PathBuf,
}

pub fn command(state: &mut State, args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let directory = normalize_path(&args.directory, &state.current_lotus_dir);

    // Get the directory node
    let dir_node = state.cache.get_dir_node(directory.to_str().unwrap());

    // Check if the directory exists
    if dir_node.is_none() {
        return Err(Box::new(PathNotFound));
    }

    // Set the current directory
    state.current_lotus_dir = directory;

    Ok(())
}
