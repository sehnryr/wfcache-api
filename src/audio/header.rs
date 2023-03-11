use anyhow::{Error, Result};

use crate::metadata::Metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioCompressionFormat {
    PCM,
    ADPCM,
    Unknown,
}

impl From<u32> for AudioCompressionFormat {
    fn from(value: u32) -> Self {
        match value {
            0x01 => AudioCompressionFormat::PCM,
            0x02 => AudioCompressionFormat::ADPCM,
            _ => AudioCompressionFormat::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct ADPCMHeader {
    pub samples_per_second: u32,
    pub bits_per_sample: u8,
    pub channels: u8,
    pub average_bytes_per_second: u32,
    pub block_align: u16,
    pub samples_per_block: u16,
    pub coefficients: [[i16; 2]; 7],
    pub size: u32,
}

impl ADPCMHeader {
    pub fn from_with_metadata<T: Into<Vec<u8>>>(data: T, metadata: Metadata) -> Result<Self> {
        let data = data.into();
        let data = data[metadata.size..].to_vec();
        let mut data_offset = 0;

        // Skip unknown u32
        data_offset += 4;

        // Read the format tag (audio compression format)
        let format_tag = AudioCompressionFormat::from(u32::from_le_bytes([
            data[data_offset],
            data[data_offset + 1],
            data[data_offset + 2],
            data[data_offset + 3],
        ]));
        data_offset += 4;

        // If the format tag is not supported, return an error
        if format_tag != AudioCompressionFormat::ADPCM {
            return Err(Error::msg("Unsupported audio compression format"));
        }

        // Skip unknown 24 bytes
        data_offset += 24;

        // Read the samples per second
        let samples_per_second = u32::from_le_bytes([
            data[data_offset],
            data[data_offset + 1],
            data[data_offset + 2],
            data[data_offset + 3],
        ]);
        data_offset += 4;

        // Read the bits per sample
        let bits_per_sample = data[data_offset];
        data_offset += 1;

        // Read the channels
        let channels = data[data_offset];
        data_offset += 1;

        // Skip unknown 4 bytes
        data_offset += 4;

        // Read the average bytes per second
        let average_bytes_per_second = u32::from_le_bytes([
            data[data_offset],
            data[data_offset + 1],
            data[data_offset + 2],
            data[data_offset + 3],
        ]);
        data_offset += 4;

        // Read the block align
        let block_align = u16::from_le_bytes([data[data_offset], data[data_offset + 1]]);
        data_offset += 2;

        // Read the samples per block
        let samples_per_block = u16::from_le_bytes([data[data_offset], data[data_offset + 1]]);
        data_offset += 2;

        // Skip unknown 12 bytes
        data_offset += 12;

        // Read the size
        let size = u32::from_le_bytes([
            data[data_offset],
            data[data_offset + 1],
            data[data_offset + 2],
            data[data_offset + 3],
        ]);

        Ok(Self {
            samples_per_second,
            bits_per_sample,
            channels,
            average_bytes_per_second,
            block_align,
            samples_per_block,
            coefficients: [
                [256, 0],
                [512, -256],
                [0, 0],
                [192, 64],
                [240, 0],
                [460, -208],
                [392, -232],
            ],
            size,
        })
    }
}
