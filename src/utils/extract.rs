use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use lotus_audio_utils::Audio;
use lotus_lib::toc::Node;
use lotus_texture_utils::Texture;

use crate::shell::State;

pub fn extract_audio(state: &State, file_node: Node, output_dir: PathBuf) -> Result<()> {
    // Get the package
    let package = state.package;

    // Get the file data and file name
    let (file_data, file_name) = package.decompress_audio(&file_node)?;

    // Get the output path
    let output_path = {
        let mut output_path = output_dir.clone();
        output_path.push(file_name);
        output_path
    };

    // Create the output file
    let mut buffer = std::fs::File::create(output_path).unwrap();

    // Write the file data to the output file
    buffer.write_all(&file_data).unwrap();

    Ok(())
}

pub fn extract_texture(state: &State, file_node: Node, output_dir: PathBuf) -> Result<()> {
    // Get the package
    let package = state.package;

    // Get the output file name
    let file_name = package.get_texture_file_name(&file_node);

    // Get the output path
    let output_path = {
        let mut output_path = output_dir.clone();
        output_path.push(file_name);
        output_path
    };

    // Get the file data
    let file_data: Vec<u8> = package.decompress_texture(&file_node)?;

    // Create the output file
    let mut buffer = std::fs::File::create(output_path).unwrap();

    // Write the file data to the output file
    buffer.write_all(&file_data).unwrap();

    Ok(())
}
