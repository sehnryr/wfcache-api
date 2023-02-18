use crate::shell::{error::PathNotFound, State};
use crate::utils::{cache::read_header, path::normalize_path};
use clap::Parser;
use std::path::PathBuf;

/// Display file status
#[derive(Parser, Debug)]
pub struct Arguments {
    file: PathBuf,
}

pub fn command(state: &State, args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let file = normalize_path(&args.file, &state.current_lotus_dir);

    // Get the file node
    let file_node = state.cache.get_file_node(file.to_str().unwrap());

    // Check if the file exists
    if file_node.is_none() {
        return Err(Box::new(PathNotFound));
    }

    // Get the file node
    let file_node = file_node.unwrap();

    // Get the decompressed header file data
    let header_file_data = state.cache.decompress_data(file_node);

    // Create the header
    let header = read_header(header_file_data);

    // Print the header
    println!("{:#?}", header);

    Ok(())
}
