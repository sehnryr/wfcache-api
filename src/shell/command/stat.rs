use anyhow::{Error, Result};
use clap::Parser;
use std::path::PathBuf;

use crate::metadata::Metadata;
use crate::shell::{error::PathNotFound, State};
use crate::utils::path::normalize_path;

/// Display file status
#[derive(Parser, Debug, Clone)]
pub struct Arguments {
    file: PathBuf,
}

pub fn command(state: &State, args: Arguments) -> Result<()> {
    let file = normalize_path(&args.file, &state.current_lotus_dir);

    // Get the file node
    let file_node = state.h_cache.get_file_node(file.to_str().unwrap());

    // Check if the file exists
    if file_node.is_none() {
        return Err(Error::from(PathNotFound));
    }

    // Get the file node
    let file_node = file_node.unwrap();

    // Get the decompressed header file data
    let header_file_data = state.h_cache.decompress_data(file_node)?;

    // Create the header
    let metadata = Metadata::from(header_file_data);

    // Print the header
    println!("{:#?}", metadata);

    Ok(())
}
