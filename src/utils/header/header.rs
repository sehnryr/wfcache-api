use super::arguments::parse_arguments;
use serde_json::Value;

#[derive(Debug)]
pub enum FileType {
    Image,
    PBRMap,
    Sound,
    Unknown,
}

impl From<u32> for FileType {
    fn from(file_type: u32) -> Self {
        match file_type {
            0xA3 | 0xB8 => FileType::Image,
            0xBC => FileType::PBRMap,
            0x8B => FileType::Sound,
            _ => FileType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub file_paths: Vec<String>,
    // pub arguments: HashMap<String, String>,
    pub arguments: Value,
    pub file_type: FileType,
    pub raw_type: String,
}

impl Header {
    pub fn default() -> Header {
        Header {
            file_paths: Vec::new(),
            // arguments: HashMap::new(),
            arguments: Value::Null,
            file_type: FileType::Unknown,
            raw_type: String::new(),
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::default()
    }
}

impl<T: Into<Vec<u8>>> From<T> for Header {
    fn from(data: T) -> Self {
        let data = data.into();
        let mut header = Header::default();

        // Base offset to skip the hash
        let mut file_paths_offset = 16;

        // Read the merged file count
        let merged_file_count = u32::from_le_bytes([
            data[file_paths_offset],
            data[file_paths_offset + 1],
            data[file_paths_offset + 2],
            data[file_paths_offset + 3],
        ]);
        file_paths_offset += 4;

        // Read the file paths
        for _ in 0..merged_file_count {
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
            header.file_paths.push(path.to_string());

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
        header.arguments = parse_arguments(raw_arguments);

        // If the arguments length is > 0, then there is a trailing null byte
        if arguments_length > 0 {
            file_paths_offset += 1;
        }

        // Read the file type
        header.raw_type = format!(
            "0x{:X}",
            u32::from_le_bytes([
                data[file_paths_offset],
                data[file_paths_offset + 1],
                data[file_paths_offset + 2],
                data[file_paths_offset + 3],
            ])
        );

        header.file_type = FileType::from(u32::from_le_bytes([
            data[file_paths_offset],
            data[file_paths_offset + 1],
            data[file_paths_offset + 2],
            data[file_paths_offset + 3],
        ]));

        header
    }
}
