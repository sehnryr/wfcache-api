use anyhow::Result;
use lotus_lib::toc::FileNode;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::shell::State;

pub fn extract(state: &State, file_node: Rc<RefCell<FileNode>>, output_dir: PathBuf) -> Result<()> {
    todo!()
}
