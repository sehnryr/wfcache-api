use anyhow::{Error, Result};
use log::debug;
use lotus_lib::cache_pair::CachePair;
use lotus_lib::toc::node::Node;
use lotus_lib::toc::FileNode;
use lotus_lib::utils::{get_block_lengths, internal_decompress_post_ensmallening};
use std::cell::RefCell;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::rc::Rc;

use crate::metadata::{FileType, Metadata};
use crate::shell::State;
use crate::texture::header::TextureHeader;

pub fn extract(state: &State, file_node: Rc<RefCell<FileNode>>, output_dir: PathBuf) -> Result<()> {
    // Get the decompressed header file data
    let header_file_data = state.h_cache.decompress_data(file_node.clone())?;
    let file_node = file_node.borrow();

    // Create the header
    let metadata = Metadata::from(header_file_data.clone());

    // Check if the file is supported
    if !metadata.is_supported() {
        return Err(Error::msg(format!(
            "File is not supported: {}",
            file_node.name()
        )));
    }
    if metadata.file_type != FileType::Texture {
        return Err(Error::msg(format!(
            "File is not an image: {}",
            file_node.name()
        )));
    }

    // Create the image
    let header = TextureHeader::from_with_header(header_file_data, metadata)?;

    debug!("Header: {:?}", header);

    // Create the output file
    let mut file_name = file_node.name().to_string();
    if file_name.ends_with(".png") {
        file_name = file_name[0..file_name.len() - 4].to_string();
    }
    file_name.push_str(".dds");

    // Create the output path
    let mut output_path = output_dir.clone();
    output_path.push(file_name);

    let file_data: Vec<u8>;

    if header.f_cache_image_count > 0 {
        let f_cache = state.f_cache.unwrap();
        let file_node = f_cache.get_file_node(file_node.path()).unwrap();
        let _file_node = file_node.borrow();

        debug!("Cache offset: {}", _file_node.cache_offset() as u64);
        debug!("Cache image size: {}", _file_node.comp_len() as u64);
        debug!("Real image size: {}", header.size() as u64);
        debug!("Decompressed image size: {}", _file_node.len() as u64);

        let f_cache_offsets = header.f_cache_image_offsets.clone();

        if f_cache_offsets.len() != 0 {
            let cache_image_sub_offset = *f_cache_offsets.last().unwrap_or(&0);

            let mut f_cache_reader = File::open(f_cache.cache_path()).unwrap();
            let real_cache_image_sub_offset = get_real_cache_image_offset(
                &mut f_cache_reader,
                _file_node.cache_offset() as usize,
                cache_image_sub_offset as usize,
            )?;

            debug!("Cache image offset: {}", cache_image_sub_offset);
            debug!("Real cache image offset: {}", real_cache_image_sub_offset);

            f_cache_reader.seek(SeekFrom::Current(real_cache_image_sub_offset as i64))?;

            file_data = internal_decompress_post_ensmallening(
                _file_node.comp_len() as usize,
                header.size() as usize,
                &mut f_cache_reader,
            )?;
        } else {
            // Fall back to the old method if the cache image offsets are not present
            let _file_data = f_cache.decompress_data(file_node.clone())?;
            file_data = _file_data[_file_data.len() - header.size().._file_data.len()].to_vec();
        }
    } else {
        let b_cache = state.b_cache.unwrap();
        let file_node = b_cache.get_file_node(file_node.path()).unwrap();
        let _file_node = file_node.borrow();

        debug!("Cache offset: {}", _file_node.cache_offset() as u64);
        debug!("Cache image size: {}", _file_node.comp_len() as u64);
        debug!("Real image size: {}", header.size() as u64);
        debug!("Decompressed image size: {}", _file_node.len() as u64);

        let _file_data = b_cache.decompress_data(file_node.clone())?;
        file_data = _file_data[_file_data.len() - header.size().._file_data.len()].to_vec();
    }

    let mut buffer = std::fs::File::create(output_path).unwrap();

    buffer.write(b"DDS ").unwrap();
    header.header.write(&mut buffer).unwrap();
    if let Some(header10) = header.header10 {
        header10.write(&mut buffer).unwrap();
    }
    buffer.write_all(&file_data).unwrap();

    Ok(())
}

fn get_real_cache_image_offset(
    cache_reader: &mut File,
    cache_image_offset: usize,
    cache_image_sub_offset: usize,
) -> Result<usize> {
    cache_reader.seek(SeekFrom::Start(cache_image_offset as u64))?;

    const BLOCK_HEADER_LEN: usize = 8;

    let mut cache_offset_top: usize = 0;
    let mut cache_offset_bottom: usize = 0;

    loop {
        let (block_compressed_len, _) = get_block_lengths(cache_reader);
        cache_offset_top += block_compressed_len as usize + BLOCK_HEADER_LEN;

        if cache_offset_top >= cache_image_sub_offset {
            break;
        }

        cache_offset_bottom = cache_offset_top;
        cache_reader.seek(SeekFrom::Current(block_compressed_len as i64))?;
    }

    // Seek back to the start of the block
    cache_reader.seek(SeekFrom::Start(cache_image_offset as u64))?;

    let diff_top = cache_offset_top - cache_image_sub_offset;
    let diff_bottom = cache_image_sub_offset - cache_offset_bottom;

    if diff_top > diff_bottom {
        return Ok(cache_offset_bottom);
    } else {
        return Ok(cache_offset_top);
    }
}
