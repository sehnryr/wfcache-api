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
    #[arg(short, default_value = "false")]
    recursive: bool,
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

    // Create output directory
    debug!("Output directory: {:?}", output_dir);
    std::fs::create_dir_all(output_dir.clone()).unwrap();

    // Extract the file or directory
    if is_file {
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
    std::fs::create_dir_all(output_dir.clone()).unwrap();

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
