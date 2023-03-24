use anyhow::Result;
use clap::Parser;
use lotus_lib::toc::node::Node;
use std::path::PathBuf;

use crate::shell::State;
use crate::utils::path::normalize_path;

enum NodeKind {
    File,
    Directory,
}

/// List the content of a directory
#[derive(Parser, Debug, Clone)]
pub struct Arguments {
    #[arg(default_value = ".")]
    directory: PathBuf,
}

pub fn command(state: &State, args: Arguments) -> Result<()> {
    let directory = normalize_path(&args.directory, &state.current_lotus_dir);

    // Get the directory node
    let dir_node = state
        .h_cache
        .get_directory_node(directory.to_str().unwrap());

    // Check if the directory exists
    if dir_node.is_none() {
        println!("Path not found: {}", directory.to_str().unwrap());
        return Ok(());
    }

    // Get the directory node
    let dir_node = dir_node.unwrap();
    let dir_node = dir_node.borrow();

    // List of nodes
    let mut nodes: Vec<(NodeKind, String)> = Vec::new();

    // Add directories
    for child_directory in dir_node.children_directories() {
        nodes.push((NodeKind::Directory, child_directory.borrow().name()));
    }

    // Add files
    for child_file in dir_node.children_files() {
        nodes.push((NodeKind::File, child_file.borrow().name()));
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
