use crate::shell::{error::PathNotFound, State};
use crate::utils::path::normalize_path;
use clap::Parser;
use std::path::PathBuf;

enum NodeKind {
    File,
    Directory,
}

/// List the content of a directory
#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(default_value = ".")]
    directory: PathBuf,
}

pub fn command(state: &State, args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let directory = normalize_path(&args.directory, &state.current_lotus_dir);

    // Get the directory node
    let dir_node = state.h_cache.get_dir_node(directory.to_str().unwrap());

    // Check if the directory exists
    if dir_node.is_none() {
        return Err(Box::new(PathNotFound));
    }

    // Get the directory node
    let dir_node = dir_node.unwrap();
    let dir_node = dir_node.borrow();

    // List of nodes
    let mut nodes: Vec<(NodeKind, String)> = Vec::new();

    // Add directories
    for (name, _) in dir_node.child_dirs() {
        nodes.push((NodeKind::Directory, name.to_string()));
    }

    // Add files
    for (name, _) in dir_node.child_files() {
        nodes.push((NodeKind::File, name.to_string()));
    }

    for (node_kind, name) in nodes {
        println!(
            "{} \t{}",
            match node_kind {
                NodeKind::File => "[f]",
                NodeKind::Directory => "[d]",
            },
            name,
        );
    }
    Ok(())
}
