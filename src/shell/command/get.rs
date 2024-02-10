use std::path::PathBuf;

use anyhow::{Error, Result};
use clap::Parser;
use log::{debug, warn};

use crate::shell::{error::PathNotFound, State};
use crate::utils::extract::{extract_dir, extract_file, extract_raw_file};
use crate::utils::path::normalize_path;

/// Extract a file from the cache or a directory recursively
#[derive(Parser, Debug, Clone)]
pub struct Arguments {
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Extract recursively
    #[arg(short, default_value = "false", conflicts_with = "cache")]
    recursive: bool,

    /// Extract without decompressing the data
    #[arg(long, default_value = "false", hide = true, requires = "cache")]
    raw: bool,

    /// Specify a cache to extract from
    #[arg(short, long, requires = "raw", hide = true, value_parser = ["H", "F", "B"], ignore_case = true)]
    cache: Option<String>,
}

pub fn command(state: &mut State, args: Arguments) -> Result<()> {
    let path = normalize_path(&args.path, &state.current_lotus_dir);
    let mut output_dir = path.strip_prefix("/").unwrap().to_path_buf();

    // Get the file node or directory node
    let file_node = state.h_cache.get_file_node(path.to_str().unwrap());
    let dir_node = state.h_cache.get_directory_node(path.to_str().unwrap());

    // Check if the file or directory exists
    let is_file = file_node.is_some();
    let is_dir = dir_node.is_some();

    if !is_file && !is_dir {
        return Err(Error::from(PathNotFound));
    }
    if is_file {
        output_dir.pop();
    }

    debug!("Output directory: {:?}", output_dir);

    // Extract the raw file if the raw flag is set
    if args.raw {
        let cache_name = args.cache.unwrap_or_else(|| unreachable!());
        return extract_raw_file(state, path, output_dir, cache_name);
    }

    // Extract the file or directory
    if is_file {
        output_dir = state.output_dir.join(output_dir);
        let file_path = file_node.clone().unwrap().path();
        match extract_file(state, file_node.unwrap(), output_dir) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Error ({}): {}", file_path.display(), e);
                Ok(())
            }
        }
    } else {
        output_dir.pop();
        output_dir = state.output_dir.join(output_dir);
        extract_dir(state, dir_node.unwrap(), output_dir, args.recursive)
    }
}
