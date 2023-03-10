use anyhow::{Error, Result};
use clap::Parser;
use lotus_lib::toc::node::Node;
use lotus_lib::toc::DirectoryNode;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::shell::{error::PathNotFound, State};
use crate::utils::path::normalize_path;

#[derive(Parser, Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    File,
    Directory,
}

impl std::str::FromStr for NodeKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(NodeKind::File),
            "directory" => Ok(NodeKind::Directory),
            _ => Err(Error::msg(format!("Invalid node type: {}", s))),
        }
    }
}

/// Find a file or directory
#[derive(Parser, Debug, Clone)]
pub struct Arguments {
    /// The directory to search in
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// Recursively search the directory
    #[arg(short, long)]
    recursive: bool,

    /// The name of the node to search for (supports wildcards)
    #[arg(short, long)]
    name: String,

    /// The type of node to search for
    #[arg(short, long)]
    type_: Option<NodeKind>,
}

pub fn command(state: &State, args: Arguments) -> Result<()> {
    let directory = normalize_path(&args.directory, &state.current_lotus_dir);

    // Get the directory node
    let dir_node = state
        .h_cache
        .get_directory_node(directory.to_str().unwrap());

    // Check if the directory exists
    if dir_node.is_none() {
        return Err(Error::from(PathNotFound));
    }

    // Get the directory node
    let dir_node = dir_node.unwrap();

    internal_find(state, dir_node, &args);

    Ok(())
}

fn internal_find(state: &State, dir_node: Rc<RefCell<DirectoryNode>>, args: &Arguments) {
    // Get the directory node
    let dir_node = dir_node.borrow();

    // List of nodes
    let mut nodes: Vec<(NodeKind, PathBuf)> = Vec::new();

    // Add directories
    for child_directory in dir_node.children_directories() {
        // Split name by '*'
        let name_parts: Vec<&str> = args.name.split('*').collect();

        // Check if the name matches
        let mut name_matches = true;
        for (i, name_part) in name_parts.iter().enumerate() {
            if i == 0 && !child_directory.borrow().name().starts_with(name_part) {
                name_matches = false;
                break;
            } else if i == name_parts.len() - 1
                && !child_directory.borrow().name().ends_with(name_part)
            {
                name_matches = false;
                break;
            } else if !child_directory.borrow().name().contains(name_part) {
                name_matches = false;
                break;
            }
        }

        if name_matches {
            nodes.push((NodeKind::Directory, child_directory.borrow().path()));
        }
        internal_find(state, child_directory, args);
    }

    // Add files
    for child_file in dir_node.children_files() {
        // Split name by '*'
        let name_parts: Vec<&str> = args.name.split('*').collect();

        // Check if the name matches
        let mut name_matches = true;
        for (i, name_part) in name_parts.iter().enumerate() {
            if i == 0 && !child_file.borrow().name().starts_with(name_part) {
                name_matches = false;
                break;
            } else if i == name_parts.len() - 1 && !child_file.borrow().name().ends_with(name_part)
            {
                name_matches = false;
                break;
            } else if !child_file.borrow().name().contains(name_part) {
                name_matches = false;
                break;
            }
        }

        if name_matches {
            nodes.push((NodeKind::File, child_file.borrow().path()));
        }
    }

    for (node_kind, path) in nodes {
        if args.type_.is_some() && args.type_.unwrap() != node_kind {
            continue;
        }

        println!("{}", path.display());
    }
}
