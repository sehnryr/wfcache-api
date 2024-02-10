use std::path::PathBuf;

use anyhow::{Error, Result};
use clap::Parser;
use lotus_lib::toc::{DirectoryNode, Node, NodeKind as LotusNodeKind};

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

    // Trim the name of any quotes
    let name = args.name.trim_matches('\'').trim_matches('"');

    internal_find(state, dir_node, name, args.recursive, args.type_);

    Ok(())
}

fn internal_find(
    state: &State,
    dir_node: Node,
    name: &str,
    recursive: bool,
    kind: Option<NodeKind>,
) {
    // Split name by '*'
    let name_parts: Vec<&str> = name.split('*').collect();

    // Add directories and files
    for child in dir_node.children() {
        // Check if the name matches
        let mut name_matches = true;
        for (i, name_part) in name_parts.iter().enumerate() {
            if i == 0 && !child.name().starts_with(name_part) {
                name_matches = false;
                break;
            } else if i == name_parts.len() - 1 && !child.name().ends_with(name_part) {
                name_matches = false;
                break;
            } else if !child.name().contains(name_part) {
                name_matches = false;
                break;
            }
        }

        let child_kind = match child.kind() {
            LotusNodeKind::Directory => NodeKind::Directory,
            LotusNodeKind::File => NodeKind::File,
        };

        if name_matches {
            if kind.is_none() || (kind.is_some() && kind.unwrap() == child_kind) {
                println!("{}", child.path().display());
            }
        }

        if child_kind == NodeKind::Directory && recursive {
            internal_find(state, child, name, recursive, kind);
        }
    }
}
