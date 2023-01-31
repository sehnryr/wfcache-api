use crate::utils::{filetime_to_unix_timestamp, get_timestamp_str};
use log::{error, trace};
use lotus_lib::cache_pair::CachePairReader;

pub fn ls(header: &CachePairReader, path: &str) {
    if !path.starts_with("/") {
        error!("Path must start with a slash: {}", path);
        std::process::exit(1);
    }

    let node = header.get_dir_node(path);

    if node.is_none() {
        error!("Directory not found: {}", path);
        std::process::exit(1);
    }

    let node = node.unwrap();
    let node = node.borrow();

    let mut nodes: Vec<(i32, i32, i64, String)> = Vec::new();

    trace!("Getting directory nodes in the root directory");
    for (name, _) in node.child_dirs() {
        nodes.push((0, 0, 0, name.to_string()));
    }

    let mut max_len_str_len = 0;
    let mut max_comp_len_str_len = 0;

    trace!("Printing files in the root directory");
    for (name, file) in node.child_files() {
        let file = file.borrow();
        nodes.push((
            file.len(),
            file.comp_len(),
            file.timestamp(),
            name.to_string(),
        ));

        let len_str_len = file.len().to_string().len();
        if len_str_len > max_len_str_len {
            max_len_str_len = len_str_len;
        }

        let comp_len_str_len = file.comp_len().to_string().len();
        if comp_len_str_len > max_comp_len_str_len {
            max_comp_len_str_len = comp_len_str_len;
        }
    }
    trace!("Max len str len: {}", max_len_str_len);
    trace!("Max comp len str len: {}", max_comp_len_str_len);

    trace!("Print the nodes");
    for (len, comp_len, timestamp, name) in nodes {
        let mut timestamp_str = timestamp.to_string();
        if timestamp > 0 {
            let timestamp = filetime_to_unix_timestamp(timestamp as u64);
            timestamp_str = get_timestamp_str(timestamp);
        }
        println!(
            "{:width$} {:comp_width$} {:>12} {}",
            len,
            comp_len,
            timestamp_str,
            name,
            width = max_len_str_len,
            comp_width = max_comp_len_str_len
        );
    }
}
