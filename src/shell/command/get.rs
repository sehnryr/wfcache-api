use crate::shell::error::MissingArgument;
use crate::shell::{error::PathNotFound, State};
use crate::utils::path::normalize_path;
use clap::Parser;
use lotus_lib::toc::{FileNode, DirNode};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// Extract a file from the cache or a directory recursively
#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Extract recursively
    #[arg(short)]
    recursive: bool,
}

pub fn command(state: &mut State, args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let path = normalize_path(&args.path, &state.current_lotus_dir);

    // Get the file node or directory node
    let file_node = state.cache.get_file_node(path.to_str().unwrap());
    let dir_node = state.cache.get_dir_node(path.to_str().unwrap());

    // Check if the file or directory exists
    let is_file = file_node.is_some();
    let is_dir = dir_node.is_some();

    if !is_file && !is_dir {
        return Err(Box::new(PathNotFound));
    }
    if !is_file && is_dir && !args.recursive {
        return Err(Box::new(MissingArgument));
    }

    // Extract the file or directory
    if is_file {
        Ok(extract_file(state, file_node.unwrap()))
    } else {
        Ok(extract_dir(state, dir_node.unwrap()))
    }
}

fn extract_file(state: &mut State, file_node: Rc<RefCell<FileNode>>) -> () {
    todo!("Extract file");
}

fn extract_dir(state: &mut State, dir_node: Rc<RefCell<DirNode>>) -> () {
    todo!("Extract directory");
}