use super::State;
use clap::Parser;
use relative_path::RelativePathBuf;
use std::path::PathBuf;

enum NodeKind {
    File,
    Directory,
}

/// List the content of a directory
#[derive(Parser, Debug)]
pub struct LsArguments {
    #[arg(default_value = ".")]
    directory: PathBuf,
}

pub fn ls_command(state: &State, args: LsArguments) -> Result<(), Box<dyn std::error::Error>> {
    let mut directory = args.directory;

    // Normalize the path
    if !directory.is_absolute() {
        // Absolute bs
        directory = state.current_lotus_dir.join(directory);
        directory = RelativePathBuf::from(directory.to_str().unwrap())
            .normalize()
            .to_path("");
        directory = PathBuf::from("/").join(directory);
    }

    // Get the directory node
    let dir_node = state.cache.get_dir_node(directory.to_str().unwrap());

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
        nodes.push((
            NodeKind::File,
            name.to_string(),
        ));
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

#[derive(Debug)]
struct PathNotFound;

impl std::fmt::Display for PathNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Path not found")
    }
}

impl std::error::Error for PathNotFound {}
