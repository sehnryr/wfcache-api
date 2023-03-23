use anyhow::{Error, Result};

use crate::metadata::Metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCompressionFormat {
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
pub struct AudioHeader {
    pub format_tag: AudioCompressionFormat,
    pub samples_per_second: u32,
    pub bits_per_sample: u8,
    pub channels: u8,
    pub average_bytes_per_second: u32,
    pub block_align: u16,
    pub samples_per_block: u16,
    pub size: u32,
}

impl AudioHeader {
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

        if format_tag == AudioCompressionFormat::Unknown {
            return Err(Error::msg("Unknown audio compression format"));
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
            format_tag,
            samples_per_second,
            bits_per_sample,
            channels,
            average_bytes_per_second,
            block_align,
            samples_per_block,
            size,
        })
    }

    pub fn to_wav_adpcm(self) -> Result<Vec<u8>> {
        let mut data = Vec::new();

        data.extend_from_slice(&[0x52, 0x49, 0x46, 0x46]); // "RIFF"
        data.extend_from_slice(&(self.size + 66).to_le_bytes()); // Size of the file minus 12 bytes
        data.extend_from_slice(&[0x57, 0x41, 0x56, 0x45]); // "WAVE"
        data.extend_from_slice(&[0x66, 0x6d, 0x74, 0x20]); // "fmt "
        data.extend_from_slice(&50u32.to_le_bytes()); // Size of the format chunk
        data.extend_from_slice(&0x02u16.to_le_bytes()); // Format tag
        data.extend_from_slice(&(self.channels as u16).to_le_bytes()); // Channels
        data.extend_from_slice(&self.samples_per_second.to_le_bytes()); // Samples per second
        data.extend_from_slice(&self.average_bytes_per_second.to_le_bytes()); // Average bytes per second
        data.extend_from_slice(&self.block_align.to_le_bytes()); // Block align
        data.extend_from_slice(&(self.bits_per_sample as u16).to_le_bytes()); // Bits per sample
        data.extend_from_slice(&32u16.to_le_bytes()); // Size of the extension
        data.extend_from_slice(&self.samples_per_block.to_le_bytes()); // Samples per block
        data.extend_from_slice(&7u16.to_le_bytes()); // Number of coefficients
        for coefficient in [
            [256, 0],
            [512, -256],
            [0, 0],
            [192, 64],
            [240, 0],
            [460, -208],
            [392, -232],
        ].iter() {
            data.extend_from_slice(&(coefficient[0] as i16).to_le_bytes()); // Coefficient 1
            data.extend_from_slice(&(coefficient[1] as i16).to_le_bytes()); // Coefficient 2
        }
        data.extend_from_slice(&[0x64, 0x61, 0x74, 0x61]); // "data"
        data.extend_from_slice(&self.size.to_le_bytes()); // Size of the data chunk

        Ok(data)
    }
}
