use anyhow::{Error, Result};
use clap::Parser;
use log::{debug, info, warn};
use lotus_lib::cache_pair::CachePair;
use lotus_lib::toc::node::Node;
use lotus_lib::toc::{DirectoryNode, FileNode};
use lotus_lib::utils::internal_decompress_post_ensmallening;
use std::cell::RefCell;
use std::io::{Seek, Write};
use std::path::PathBuf;
use std::rc::Rc;

use crate::shell::{error::PathNotFound, State};
use crate::utils::header::{FileType, Header, Image};
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
    match if is_file {
        extract_file(state, file_node.unwrap(), output_dir)
    } else {
        output_dir.pop();
        extract_dir(state, dir_node.unwrap(), output_dir, args.recursive)
    } {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("Error: {}", e);
            Err(e)
        }
    }
}

fn extract_file(
    state: &State,
    file_node: Rc<RefCell<FileNode>>,
    output_dir: PathBuf,
) -> Result<()> {
    // Get the decompressed header file data
    let header_file_data = state.h_cache.decompress_data(file_node.clone())?;
    let file_node = file_node.borrow();

    // Create the header
    let header = Header::from(header_file_data.clone());

    // Check if the file is supported
    if !header.is_supported() {
        return Err(Error::msg(format!(
            "File is not supported: {}",
            file_node.name()
        )));
    }
    if ![FileType::Image, FileType::PBRMap].contains(&header.file_type) {
        return Err(Error::msg(format!(
            "File is not an image: {}",
            file_node.name()
        )));
    }

    // Create the image
    let header = Image::from_with_header(header_file_data, header);

    debug!("Header: {:?}", header);

    // Create the output file
    let mut file_name = file_node.name().to_string();
    if file_name.ends_with(".png") {
        file_name = file_name[0..file_name.len() - 4].to_string();
    }
    file_name.push_str(".dds");

    // Create the output path
    let mut output_path = output_dir.clone();
    output_path.push(file_name);

    let file_data: Vec<u8>;

    if header.f_cache_image_count > 0 {
        let cache_image_offset = *header.f_cache_image_offsets.last().unwrap_or(&0);
        let f_cache = state.f_cache.unwrap();
        let file_node = f_cache.get_file_node(file_node.path()).unwrap();
        let file_node = file_node.borrow();
        let mut f_cache_reader = std::fs::File::open(f_cache.cache_path()).unwrap();

        debug!("Cache offset: {}", file_node.cache_offset() as u64);
        debug!("Cache image size: {}", file_node.comp_len() as u64);
        debug!("Real image size: {}", header.size() as u64);

        f_cache_reader
            .seek(std::io::SeekFrom::Start(
                file_node.cache_offset() as u64 + cache_image_offset as u64,
            ))
            .unwrap();

        file_data = internal_decompress_post_ensmallening(
            file_node.comp_len() as usize,
            header.size(),
            &mut f_cache_reader,
        )?;
    } else {
        let b_cache = state.b_cache.unwrap();
        let file_node = b_cache.get_file_node(file_node.path()).unwrap();
        let _file_node = file_node.borrow();

        debug!("Cache offset: {}", _file_node.cache_offset() as u64);
        debug!("Cache image size: {}", _file_node.comp_len() as u64);
        debug!("Real image size: {}", header.size() as u64);

        let _file_data = b_cache.decompress_data(file_node.clone())?;
        file_data = _file_data[_file_data.len() - header.size().._file_data.len()].to_vec();
    }

    let mut buffer = std::fs::File::create(output_path).unwrap();

    buffer.write(b"DDS ").unwrap();
    header.header.write(&mut buffer).unwrap();
    buffer.write_all(&file_data).unwrap();

    Ok(())
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
        info!(
            "Extracting file: {}",
            file_child_node.borrow().path().display()
        );
        extract_file(state, file_child_node.clone(), output_dir.clone())?;
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
