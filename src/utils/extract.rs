use std::io::Write;
use std::path::PathBuf;

use anyhow::{Error, Result};
use log::{debug, info, warn};
use lotus_audio_utils::Audio;
use lotus_lib::toc::{DirectoryNode, Node, NodeKind};
use lotus_texture_utils::Texture;

use crate::metadata::{FileType, Metadata};
use crate::shell::State;

pub fn extract_file(state: &State, file_node: Node, output_dir: PathBuf) -> Result<()> {
    let header_file_data = state.h_cache.decompress_data(file_node.clone())?;
    let metadata = Metadata::from(header_file_data.clone());

    if metadata.is_supported() {
        std::fs::create_dir_all(output_dir.clone()).unwrap();
    }

    match metadata.file_type {
        FileType::Audio => extract_audio(state, file_node, output_dir),
        FileType::Texture => extract_texture(state, file_node, output_dir),
        _ => Err(Error::msg(format!(
            "File is not supported: {}",
            file_node.name()
        ))),
    }
}

pub fn extract_dir(
    state: &State,
    dir_node: Node,
    output_dir: PathBuf,
    recursive: bool,
) -> Result<()> {
    // Create the output directory
    let mut output_dir = output_dir.clone();
    output_dir.push(dir_node.name());

    // Extract the files
    for child_node in dir_node.children() {
        if child_node.kind() != NodeKind::File {
            continue;
        }

        let file_path = child_node.path();
        info!("Extracting file: {}", file_path.display());
        match extract_file(state, child_node.clone(), output_dir.clone()) {
            Ok(_) => {}
            Err(e) => warn!("Error ({}): {}", file_path.display(), e),
        }
    }

    // Extract the directories
    if recursive {
        for child_node in dir_node.children() {
            if child_node.kind() != NodeKind::Directory {
                continue;
            }

            debug!("Extracting directory: {}", child_node.name());
            extract_dir(state, child_node.clone(), output_dir.clone(), recursive)?;
        }
    }

    Ok(())
}

pub fn extract_raw_file(
    state: &State,
    file_path: PathBuf,
    output_dir: PathBuf,
    cache_name: String,
) -> Result<()> {
    let cache = match cache_name.to_uppercase().as_str() {
        "H" => Some(state.h_cache),
        "F" => state.f_cache,
        "B" => state.b_cache,
        _ => unreachable!(),
    };

    // Check if the cache exists
    if cache.is_none() {
        return Err(Error::msg(format!("Cache '{}' does not exist", cache_name)));
    }

    let cache = cache.unwrap();

    // Get the file node
    let file_node = cache.get_file_node(file_path.to_str().unwrap());

    // Check if the file exists in the cache
    if file_node.is_none() {
        warn!("File '{}' does not exist in the cache", file_path.display());
        return Ok(());
    }

    // Extract the file
    let file_node = file_node.unwrap();
    let file_data = cache.get_data(file_node.clone())?;

    // Create the output directory
    std::fs::create_dir_all(output_dir.clone()).unwrap();

    // Write the file
    let file_path = output_dir.join(format!("{}.{}.raw", file_node.name(), cache_name));
    std::fs::write(file_path.clone(), file_data)?;

    info!("Extracted file to '{}'", file_path.display());

    Ok(())
}

fn extract_audio(state: &State, file_node: Node, output_dir: PathBuf) -> Result<()> {
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

fn extract_texture(state: &State, file_node: Node, output_dir: PathBuf) -> Result<()> {
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
