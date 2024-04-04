use std::io::{Error, ErrorKind, Result, Write};
use std::path::PathBuf;

use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::package::Package;
use lotus_lib::toc::{DirectoryNode, Node, NodeKind};
use lotus_utils_audio::Audio;
use lotus_utils_texture::Texture;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub fn extract_file(
    package: &Package<CachePairReader>,
    file_node: &Node,
    output_dir: &PathBuf,
    count: usize,
    total: usize,
    progress_tx: UnboundedSender<(usize, usize)>,
) -> Result<()> {
    progress_tx.send((count, total)).unwrap();

    let file_name = file_node.name();
    let file_path = file_node.path();

    let output_dir = if let Some(parent) = file_path.strip_prefix("/").unwrap().parent() {
        let mut output_dir = output_dir.clone();
        output_dir.push(parent);
        output_dir
    } else {
        output_dir.clone()
    };
    std::fs::create_dir_all(&output_dir).unwrap();

    if file_name.ends_with(".png") {
        extract_texture(package, file_node, &output_dir)?;
    } else if file_name.ends_with(".wav") {
        extract_audio(package, file_node, &output_dir)?;
    } else {
        extract_decompressed(package, &file_path, &output_dir, "H")?;
    }

    progress_tx.send((count + 1, total)).unwrap();

    Ok(())
}

pub fn extract_dir(
    package: &Package<CachePairReader>,
    dir_node: &Node,
    output_dir: &PathBuf,
    recursive: bool,
    extract_rx: &mut UnboundedReceiver<()>,
    progress_tx: UnboundedSender<(usize, usize)>,
) -> Result<()> {
    let mut files: Vec<Node> = Vec::new();
    let mut directories: Vec<Node> = vec![dir_node.clone()];

    while directories.len() > 0 {
        let directory = directories.remove(0);
        for child_node in directory.children() {
            if child_node.kind() == NodeKind::Directory && recursive {
                directories.push(child_node);
            } else if child_node.kind() == NodeKind::File {
                files.push(child_node);
            }
        }
    }

    let total = files.len();

    for (count, file_node) in files.iter().enumerate() {
        if extract_rx.try_recv().is_ok() {
            break;
        }

        extract_file(
            package,
            file_node,
            &output_dir,
            count,
            total,
            progress_tx.clone(),
        )?;
    }

    Ok(())
}

/// Decompress and extract a file from the cache without parsing it (e.g. audio, texture)
pub fn extract_decompressed(
    package: &Package<CachePairReader>,
    file_path: &PathBuf,
    output_dir: &PathBuf,
    cache_name: &str,
) -> Result<()> {
    let cache = package.borrow(cache_name);

    // Check if the cache exists
    let cache = cache.ok_or(Error::new(
        ErrorKind::NotFound,
        format!("Cache {} not found", cache_name),
    ))?;

    // Get the file node
    let file_node = cache.get_file_node(&file_path).ok_or(Error::new(
        ErrorKind::NotFound,
        format!("File '{}' does not exist in the cache", file_path.display()),
    ))?;

    // Extract the file
    let file_data = cache.decompress_data(file_node.clone()).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to decompress file '{}': {}", file_node.name(), e),
        )
    })?;

    // Write the file
    std::fs::write(output_dir.join(file_path.file_name().unwrap()), file_data)?;

    Ok(())
}

fn extract_audio(
    package: &Package<CachePairReader>,
    file_node: &Node,
    output_dir: &PathBuf,
) -> Result<()> {
    // Get the file data and file name
    let (file_data, file_name) = package.decompress_audio(file_node).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to decompress audio '{}': {}", file_node.name(), e),
        )
    })?;

    // Create the output file
    let mut buffer = std::fs::File::create(output_dir.join(file_name)).unwrap();

    // Write the file data to the output file
    buffer.write_all(&file_data).unwrap();

    Ok(())
}

fn extract_texture(
    package: &Package<CachePairReader>,
    file_node: &Node,
    output_dir: &PathBuf,
) -> Result<()> {
    let file_name = package.get_texture_file_name(file_node);
    let file_data: Vec<u8> = package.decompress_texture(file_node).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to decompress texture '{}': {}", file_node.name(), e),
        )
    })?;

    // Create the output file
    let mut buffer = std::fs::File::create(output_dir.join(file_name)).unwrap();

    // Write the file data to the output file
    buffer.write_all(&file_data).unwrap();

    Ok(())
}
