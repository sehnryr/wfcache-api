use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum DDSFormat {
    Default = 0x00,
    DXT1 = 0x01,
    DXT3 = 0x02,
    DXT5 = 0x03,
    ATI1 = 0x06,
    ATI2 = 0x07,
}

#[derive(Debug)]
pub struct HeaderPath {
    pub length: u32,
    pub path: String,
}

#[derive(Debug)]
pub struct Header {
    pub merged_file_count: u32,
    pub file_paths: Vec<HeaderPath>,

    pub arguments_count: u32,
    pub arguments: HashMap<String, String>,

    pub file_type: u32,

    pub f_cache_image_count: u8,
    pub f_cache_image_offsets: Vec<u32>,

    pub dds_format: DDSFormat,
    pub dds_width_px: u32,
    pub dds_height_px: u32,
    pub dds_size: u64,
}

impl Header {
    pub fn default() -> Header {
        Header {
            merged_file_count: 0,
            file_paths: Vec::new(),
            arguments_count: 0,
            arguments: HashMap::new(),
            file_type: 0,
            f_cache_image_count: 0,
            f_cache_image_offsets: Vec::new(),
            dds_format: DDSFormat::Default,
            dds_width_px: 0,
            dds_height_px: 0,
            dds_size: 0,
        }
    }
}

pub fn dds_size(width: u32, height: u32, format: DDSFormat) -> u64 {
    let block_size: u64 = match format {
        DDSFormat::DXT1 | DDSFormat::ATI1 => 8,
        DDSFormat::DXT3 | DDSFormat::DXT5 | DDSFormat::ATI2 => 16,
        _ => 8,
    };

    let block_count = ((width + 3) / 4) as u64 * ((height + 3) / 4) as u64;

    block_count * block_size
}

pub fn read_header(data: Vec<u8>) -> Header {
    let mut header = Header::default();

    // Base offset to skip the hash
    let mut file_paths_offset = 16;

    // Read the merged file count
    header.merged_file_count = u32::from_le_bytes([
        data[file_paths_offset],
        data[file_paths_offset + 1],
        data[file_paths_offset + 2],
        data[file_paths_offset + 3],
    ]);
    file_paths_offset += 4;

    // Read the file paths
    for _ in 0..header.merged_file_count {
        // Read the path length
        let path_length = u32::from_le_bytes([
            data[file_paths_offset],
            data[file_paths_offset + 1],
            data[file_paths_offset + 2],
            data[file_paths_offset + 3],
        ]);

        // Read the path
        let path = String::from_utf8_lossy(
            &data[file_paths_offset + 4..file_paths_offset + 4 + path_length as usize],
        );

        // Add the path to the header
        header.file_paths.push(HeaderPath {
            length: path_length,
            path: path.to_string(),
        });

        // Increment the offset
        file_paths_offset += 4 + path_length as usize;
    }

    // Read the arguments length
    let arguments_length = u32::from_le_bytes([
        data[file_paths_offset],
        data[file_paths_offset + 1],
        data[file_paths_offset + 2],
        data[file_paths_offset + 3],
    ]);
    file_paths_offset += 4;

    // Read the arguments
    let raw_arguments = String::from_utf8_lossy(
        &data[file_paths_offset..file_paths_offset + arguments_length as usize],
    )
    .to_string();
    file_paths_offset += arguments_length as usize;

    // Parse the arguments
    for argument in raw_arguments.split('\n') {
        let argument_split = argument.split_once('=');
        if argument_split.is_none() {
            continue;
        }
        let argument_split = argument_split.unwrap();

        let key = argument_split.0;
        let value = argument_split.1;

        header.arguments_count += 1;
        header.arguments.insert(key.to_string(), value.to_string());
    }

    // If the arguments length is > 0, then there is a trailing null byte
    if arguments_length > 0 {
        file_paths_offset += 1;
    }

    // Read the file type
    header.file_type = u32::from_le_bytes([
        data[file_paths_offset],
        data[file_paths_offset + 1],
        data[file_paths_offset + 2],
        data[file_paths_offset + 3],
    ]);
    file_paths_offset += 4;

    // Skip unknown byte
    file_paths_offset += 1;

    // Read the F cache image count
    header.f_cache_image_count = data[file_paths_offset];
    file_paths_offset += 1;

    // Skip unknown byte
    file_paths_offset += 1;

    // Read the DDS format
    header.dds_format = match data[file_paths_offset] {
        0x01 => DDSFormat::DXT1,
        0x02 => DDSFormat::DXT3,
        0x03 => DDSFormat::DXT5,
        0x06 => DDSFormat::ATI1,
        0x07 => DDSFormat::ATI2,
        _ => DDSFormat::Default,
    };
    file_paths_offset += 1;

    // Skip the mip map count
    file_paths_offset += 4;

    // Read the F cache image offsets
    for _ in 0..header.f_cache_image_count {
        header.f_cache_image_offsets.push(u32::from_le_bytes([
            data[file_paths_offset],
            data[file_paths_offset + 1],
            data[file_paths_offset + 2],
            data[file_paths_offset + 3],
        ]));
        file_paths_offset += 4;
    }

    // Read the width ratio
    let width_ratio = u16::from_le_bytes([data[file_paths_offset], data[file_paths_offset + 1]]);
    file_paths_offset += 2;

    // Read the height ratio
    let height_ratio = u16::from_le_bytes([data[file_paths_offset], data[file_paths_offset + 1]]);
    file_paths_offset += 2;

    // Skip the B cache max width
    file_paths_offset += 2;

    // Skip the B cache max height
    file_paths_offset += 2;

    // Read the DDS max side size
    let dds_max_side_size = u32::from_le_bytes([
        data[file_paths_offset],
        data[file_paths_offset + 1],
        data[file_paths_offset + 2],
        data[file_paths_offset + 3],
    ]);
    // file_paths_offset += 4;

    // Calculate the DDS width and height
    if width_ratio > height_ratio {
        header.dds_width_px = dds_max_side_size;
        header.dds_height_px = dds_max_side_size * height_ratio as u32 / width_ratio as u32;
    } else {
        header.dds_width_px = dds_max_side_size * width_ratio as u32 / height_ratio as u32;
        header.dds_height_px = dds_max_side_size;
    }

    // Calculate the DDS size
    header.dds_size = dds_size(header.dds_width_px, header.dds_height_px, header.dds_format);

    header
}
