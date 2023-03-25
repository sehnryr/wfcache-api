use anyhow::Result;
use clap::Parser;
use lotus_lib::toc::node::Node;
use std::path::PathBuf;
use term_grid::{Cell, Direction, Filling, Grid, GridOptions};
use terminal_size::{terminal_size, Height, Width};

use crate::shell::State;
use crate::utils::path::normalize_path;

#[derive(PartialEq, Eq)]
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

    // Get the terminal size
    let size = terminal_size();

    // Get the width of the terminal (default to 80)
    let mut width: usize = match size {
        Some((Width(w), Height(_))) => w.into(),
        None => 80,
    };

    // Create the grid
    let mut grid = Grid::new(GridOptions {
        filling: Filling::Spaces(1),
        direction: Direction::TopToBottom,
    });

    // Add the nodes to the grid
    for (_, name) in nodes {
        // Add the cell
        grid.add(Cell::from(name.clone()));

        // If the name is longer than the current width, update the width
        if name.len() > width {
            width = name.len();
        }
    }

    // Print the grid
    print!("{}", grid.fit_into_width(width).unwrap());

    Ok(())
}
