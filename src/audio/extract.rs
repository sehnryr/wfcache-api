use anyhow::{Error, Ok, Result};
use log::debug;
use lotus_lib::toc::node::Node;
use lotus_lib::toc::FileNode;
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use crate::audio::header::{AudioCompressionFormat, AudioHeader};
use crate::audio::ogg::{get_segment_table, OggPage};
use crate::metadata::Metadata;
use crate::shell::State;

pub fn extract(state: &State, file_node: Rc<RefCell<FileNode>>, output_dir: PathBuf) -> Result<()> {
    // Get the decompressed header file data
    let header_file_data = state.h_cache.decompress_data(file_node.clone())?;
    let file_node = file_node.borrow();

    // Create the metadata
    let metadata = Metadata::from(header_file_data.clone());

    // Create the audio
    let header = AudioHeader::from_with_metadata(header_file_data, metadata)?;

    debug!("Header: {:?}", header);

    // Create the output file
    let file_name = file_node.name().to_string();

    // Create the output path
    let mut output_path = output_dir.clone();
    output_path.push(file_name);

    if header.format_tag == AudioCompressionFormat::ADPCM
    {
        output_path.set_extension("wav");

        // Get the file data
        let b_cache = state.b_cache.unwrap();
        let f_cache = state.f_cache.unwrap();

        let b_file_node = b_cache.get_file_node(file_node.path());
        let f_file_node = f_cache.get_file_node(file_node.path());

        let mut file_data = Vec::new();

        if b_file_node.is_some() {
            let b_file_node = b_file_node.unwrap();
            let _b_file_node = b_file_node.borrow();

            debug!("Part B file node found!");

            debug!("Cache offset: {}", _b_file_node.cache_offset() as u64);
            debug!("Cache audio size: {}", _b_file_node.comp_len() as u64);
            debug!("Decompressed audio size: {}", _b_file_node.len() as u64);

            let b_file_data = b_cache.decompress_data(b_file_node.clone())?;
            file_data.extend_from_slice(&b_file_data);
        }

        if f_file_node.is_some() {
            let f_file_node = f_file_node.unwrap();
            let _f_file_node = f_file_node.borrow();

            debug!("Part F file node found!");

            debug!("Cache offset: {}", _f_file_node.cache_offset() as u64);
            debug!("Cache audio size: {}", _f_file_node.comp_len() as u64);
            debug!("Decompressed audio size: {}", _f_file_node.len() as u64);

            let f_file_data = f_cache.decompress_data(f_file_node.clone())?;
            file_data.extend_from_slice(&f_file_data);
        }

        debug!("Real audio size: {}", header.size as u64);

        let file_data = file_data[(file_data.len() - header.size as usize)..].to_vec();

        // Write the file
        let mut buffer = std::fs::File::create(output_path)?;

        buffer.write_all(&header.to_wav_adpcm().unwrap())?;
        buffer.write_all(&file_data)?;

        return Ok(());
    } else if header.format_tag == AudioCompressionFormat::Opus
    {
        output_path.set_extension("opus");

        // Get the file data
        let b_cache = state.b_cache.unwrap();
        let f_cache = state.f_cache.unwrap();

        let b_file_node = b_cache.get_file_node(file_node.path());
        let f_file_node = f_cache.get_file_node(file_node.path());

        let mut file_data = Vec::new();

        if f_file_node.is_some() {
            let f_file_node = f_file_node.clone().unwrap();
            let _f_file_node = f_file_node.borrow();

            debug!("Part F file node found!");

            debug!("Cache offset: {}", _f_file_node.cache_offset() as u64);
            debug!("Cache audio size: {}", _f_file_node.comp_len() as u64);
            debug!("Decompressed audio size: {}", _f_file_node.len() as u64);

            let f_file_data = f_cache.decompress_data(f_file_node.clone())?;
            file_data.extend_from_slice(&f_file_data);
        }

        if (f_file_node.is_none() || file_data.len() != header.size as usize)
            && b_file_node.is_some()
        {
            let b_file_node = b_file_node.unwrap();
            let _b_file_node = b_file_node.borrow();

            debug!("Part B file node found!");

            debug!("Cache offset: {}", _b_file_node.cache_offset() as u64);
            debug!("Cache audio size: {}", _b_file_node.comp_len() as u64);
            debug!("Decompressed audio size: {}", _b_file_node.len() as u64);

            let b_file_data = b_cache.decompress_data(b_file_node.clone())?;
            file_data.extend_from_slice(&b_file_data);
        }

        debug!("Real audio size: {}", header.size as u64);

        let file_data = file_data[..header.size as usize].to_vec();

        // Write the file
        let mut buffer = std::fs::File::create(output_path)?;

        buffer.write_all(&header.clone().to_opus().unwrap())?;

        // Write the opus data
        let mut page_sequence_number = 2;
        let mut granule_position = header.samples_per_second as u64;

        let chunk_size = header.block_align as usize * 50;

        for chunk in file_data.chunks(chunk_size) {
            let header_type = if chunk.len() < chunk_size { 0x04 } else { 0x00 };
            let segment_table = get_segment_table(chunk, header.block_align.into());
            let data_page = OggPage::new(
                header_type,
                granule_position,
                header.stream_serial_number,
                page_sequence_number,
                segment_table.len() as u8,
                segment_table,
                chunk.to_vec(),
            );

            buffer.write_all(&Into::<Vec<u8>>::into(data_page))?;

            page_sequence_number += 1;
            granule_position += header.samples_per_second as u64;
        }

        return Ok(());
    }

    Err(Error::msg("Error extracting audio file"))
}
