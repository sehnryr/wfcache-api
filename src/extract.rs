use std::path::PathBuf;

use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::package::{Package, PackageType};
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
) {
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

    let mut file_name: String = file_name;
    let file_data: Vec<u8>;

    if package.is_texture(file_node).unwrap() {
        (file_data, file_name) = package.decompress_texture(file_node).unwrap();
    } else if package.is_audio(file_node).unwrap() {
        (file_data, file_name) = package.decompress_audio(file_node).unwrap();
    } else {
        // Decompress and extract a file from the cache without parsing it (e.g. audio, texture)
        let cache = package.borrow(PackageType::H).unwrap();
        let file_node = cache.get_file_node(&file_path).unwrap();
        file_data = cache.decompress_data(file_node.clone()).unwrap()
    }

    // Write the file
    std::fs::write(output_dir.join(file_name), file_data).unwrap();

    progress_tx.send((count + 1, total)).unwrap();
}

pub fn extract_dir(
    package: &Package<CachePairReader>,
    dir_node: &Node,
    output_dir: &PathBuf,
    recursive: bool,
    extract_rx: &mut UnboundedReceiver<()>,
    progress_tx: UnboundedSender<(usize, usize)>,
) {
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
        );
    }
}
