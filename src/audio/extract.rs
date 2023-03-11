use anyhow::{Ok, Result};
use log::debug;
use lotus_lib::toc::node::Node;
use lotus_lib::toc::FileNode;
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use crate::audio::header::ADPCMHeader;
use crate::metadata::Metadata;
use crate::shell::State;

pub fn extract(state: &State, file_node: Rc<RefCell<FileNode>>, output_dir: PathBuf) -> Result<()> {
    // Get the decompressed header file data
    let header_file_data = state.h_cache.decompress_data(file_node.clone())?;
    let file_node = file_node.borrow();

    // Create the metadata
    let metadata = Metadata::from(header_file_data.clone());

    // Create the audio
    let header = ADPCMHeader::from_with_metadata(header_file_data, metadata)?;

    debug!("Header: {:?}", header);

    // Create the output file
    let file_name = file_node.name().to_string();

    // Create the output path
    let mut output_path = output_dir.clone();
    output_path.push(file_name);

    // Get the file data
    let b_cache = state.b_cache.unwrap();
    let file_node = b_cache.get_file_node(file_node.path()).unwrap();
    let _file_node = file_node.borrow();

    debug!("Cache offset: {}", _file_node.cache_offset() as u64);
    debug!("Cache audio size: {}", _file_node.comp_len() as u64);
    debug!("Real audio size: {}", header.size as u64);
    debug!("Decompressed audio size: {}", _file_node.len() as u64);

    let file_data = b_cache.decompress_data(file_node.clone())?;
    let file_data = file_data[..header.size as usize].to_vec();

    // Write the file
    let mut buffer = std::fs::File::create(output_path)?;

    buffer.write(b"RIFF")?;
    buffer.write(&(header.size + 66).to_le_bytes())?;
    buffer.write(b"WAVEfmt ")?;
    buffer.write(&50u32.to_le_bytes())?;
    buffer.write(&0x02u16.to_le_bytes())?;
    buffer.write(&(header.channels as u16).to_le_bytes())?;
    buffer.write(&header.samples_per_second.to_le_bytes())?;
    buffer.write(&header.average_bytes_per_second.to_le_bytes())?;
    buffer.write(&header.block_align.to_le_bytes())?;
    buffer.write(&(header.bits_per_sample as u16).to_le_bytes())?;
    buffer.write(&32u16.to_le_bytes())?;
    buffer.write(&header.samples_per_block.to_le_bytes())?;
    buffer.write(&(header.coefficients.len() as u16).to_le_bytes())?;
    for coefficient in header.coefficients.iter() {
        debug!("Coefficient: {:?}", coefficient);
        buffer.write(&coefficient[0].to_le_bytes())?;
        buffer.write(&coefficient[1].to_le_bytes())?;
    }
    buffer.write(b"data")?;
    buffer.write(&header.size.to_le_bytes())?;
    buffer.write_all(&file_data)?;

    Ok(())
}
