use derivative::Derivative;
use serde_json::Value;

use crate::metadata::arguments::parse_arguments;
use crate::music::MusicType;
use crate::texture::TextureType;

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Music,
    Texture,
    Unknown,
}

impl From<u32> for FileType {
    fn from(file_type: u32) -> Self {
        match file_type {
            MusicType::MUSIC_139 => FileType::Music,
            TextureType::DIFFUSE_EMISSION_TINT => FileType::Texture,
            TextureType::BILLBOARD_SPRITEMAP_DIFFUSE => FileType::Texture,
            TextureType::BILLBOARD_SPRITEMAP_NORMAL => FileType::Texture,
            TextureType::ROUGHNESS => FileType::Texture,
            TextureType::SKYBOX => FileType::Texture,
            TextureType::TEXTURE_174 => FileType::Texture,
            TextureType::TEXTURE_176 => FileType::Texture,
            TextureType::CUBEMAP => FileType::Texture,
            TextureType::NORMAL_MAP => FileType::Texture,
            TextureType::PACKMAP => FileType::Texture,
            TextureType::TEXTURE_194 => FileType::Texture,
            TextureType::DETAILSPACK => FileType::Texture,
            _ => FileType::Unknown,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Metadata {
    pub file_paths: Vec<String>,
    pub arguments: Value,
    pub file_type: FileType,
    pub raw_type: String,

    #[derivative(Debug = "ignore")]
    pub size: usize,
}

impl Metadata {
    pub fn is_supported(&self) -> bool {
        self.file_type != FileType::Unknown
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            file_paths: Vec::new(),
            arguments: Value::Null,
            file_type: FileType::Unknown,
            raw_type: String::new(),
            size: 0,
        }
    }
}

impl<T: Into<Vec<u8>>> From<T> for Metadata {
    fn from(data: T) -> Self {
        let data = data.into();
        let mut metadata = Metadata::default();

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
            metadata.file_paths.push(path.to_string());

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
        metadata.arguments = parse_arguments(raw_arguments);

        // If the arguments length is > 0, then there is a trailing null byte
        if arguments_length > 0 {
            file_paths_offset += 1;
        }

        // Read the file type
        let file_type = u32::from_le_bytes([
            data[file_paths_offset],
            data[file_paths_offset + 1],
            data[file_paths_offset + 2],
            data[file_paths_offset + 3],
        ]);
        file_paths_offset += 4;

        metadata.raw_type = format!("0x{:X}", file_type);

        metadata.file_type = FileType::from(file_type);

        // Set the size
        metadata.size = file_paths_offset;

        metadata
    }
}
