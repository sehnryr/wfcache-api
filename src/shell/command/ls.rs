use anyhow::Result;
use clap::Parser;
use lotus_lib::toc::{DirectoryNode, NodeKind};
use lscolors::{Indicator, LsColors, Style};
use std::path::PathBuf;
use term_grid::{Cell, Direction, Filling, Grid, GridOptions};
use terminal_size::{terminal_size, Height, Width};

use crate::shell::State;
use crate::utils::path::normalize_path;

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

    // Get the children of the directory and sort them by name
    let mut nodes = dir_node.children();
    nodes.sort_by(|a, b| a.name().cmp(&b.name()));

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

    // Get the lscolors
    let lscolors = LsColors::from_env().unwrap_or_default();

    // Add the nodes to the grid
    for node in nodes {
        let node_kind = node.kind();
        let node_name = node.name();

        // Get the style
        let style = match node_kind {
            NodeKind::File => lscolors.style_for_indicator(Indicator::RegularFile),
            NodeKind::Directory => lscolors.style_for_indicator(Indicator::Directory),
        };

        // Add the cell with colored name
        grid.add(Cell::from(
            style
                .map(Style::to_ansi_term_style)
                .unwrap_or_default()
                .paint(node_name.clone())
                .to_string(),
        ));

        // If the name is longer than the current width, update the width
        if node_name.len() > width {
            width = node_name.len();
        }
    }

    // Print the grid
    print!("{}", grid.fit_into_width(width).unwrap());

    Ok(())
}
