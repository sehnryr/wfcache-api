use anyhow::{Error, Result};
use clap::Parser;
use log::{debug, info, warn};
use lotus_lib::toc::node::Node;
use lotus_lib::toc::{DirectoryNode, FileNode};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::audio::extract as extract_audio;
use crate::metadata::{FileType, Metadata};
use crate::shell::{error::PathNotFound, State};
use crate::texture::extract as extract_texture;
use crate::utils::path::normalize_path;

/// Extract a file from the cache or a directory recursively
#[derive(Parser, Debug, Clone)]
pub struct Arguments {
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Extract recursively
    #[arg(short, default_value = "false", conflicts_with = "cache")]
    recursive: bool,

    /// Extract without decompressing the data
    #[arg(long, default_value = "false", hide = true, requires = "cache")]
    raw: bool,

    /// Specify a cache to extract from
    #[arg(short, long, requires = "raw", hide = true, value_parser = ["H", "F", "B"], ignore_case = true)]
    cache: Option<String>,
}

pub fn command(state: &mut State, args: Arguments) -> Result<()> {
    let path = normalize_path(&args.path, &state.current_lotus_dir);
    let mut output_dir = path.strip_prefix("/").unwrap().to_path_buf();

    // Get the file node or directory node
    let file_node = state.h_cache.get_file_node(path.to_str().unwrap());
    let dir_node = state.h_cache.get_directory_node(path.to_str().unwrap());

    // Check if the file or directory exists
    let is_file = file_node.is_some();
    let is_dir = dir_node.is_some();

    if !is_file && !is_dir {
        return Err(Error::from(PathNotFound));
    }
    if is_file {
        output_dir.pop();
    }

    debug!("Output directory: {:?}", output_dir);

    // Extract the raw file if the raw flag is set
    if args.raw {
        return extract_raw_file(state, args, path, output_dir);
    }

    // Extract the file or directory
    if is_file {
        output_dir = state.output_dir.join(output_dir);
        let file_path = file_node.clone().unwrap().borrow().path();
        match extract_file(state, file_node.unwrap(), output_dir) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Error ({}): {}", file_path.display(), e);
                Ok(())
            }
        }
    } else {
        output_dir.pop();
        output_dir = state.output_dir.join(output_dir);
        extract_dir(state, dir_node.unwrap(), output_dir, args.recursive)
    }
}

fn extract_file(
    state: &State,
    file_node: Rc<RefCell<FileNode>>,
    output_dir: PathBuf,
) -> Result<()> {
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
            file_node.borrow().name()
        ))),
    }
}

fn extract_dir(
    state: &State,
    dir_node: Rc<RefCell<DirectoryNode>>,
    output_dir: PathBuf,
    recursive: bool,
) -> Result<()> {
    let dir_node = dir_node.borrow();

    // Create the output directory
    let mut output_dir = output_dir.clone();
    output_dir.push(dir_node.name());

    // Extract the files
    for file_child_node in dir_node.children_files() {
        let file_path = file_child_node.borrow().path();
        info!("Extracting file: {}", file_path.display());
        match extract_file(state, file_child_node.clone(), output_dir.clone()) {
            Ok(_) => {}
            Err(e) => warn!("Error ({}): {}", file_path.display(), e),
        }
    }

    // Extract the directories
    if recursive {
        for dir_child_node in dir_node.children_directories() {
            debug!("Extracting directory: {}", dir_child_node.borrow().name());
            extract_dir(state, dir_child_node.clone(), output_dir.clone(), recursive)?;
        }
    }

    Ok(())
}

fn extract_raw_file(
    state: &State,
    args: Arguments,
    file_path: PathBuf,
    output_dir: PathBuf,
) -> Result<()> {
    let cache_name = args.cache.unwrap();
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
    let file_path = output_dir.join(format!("{}.{}.raw", file_node.borrow().name(), cache_name));
    std::fs::write(file_path.clone(), file_data)?;

    info!("Extracted file to '{}'", file_path.display());

    Ok(())
}
