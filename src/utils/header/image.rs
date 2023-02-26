use ddsfile::{DxgiFormat, FourCC, Header as DDSHeader};
use derivative::Derivative;
use std::cmp::max;

use crate::utils::header::Header;

fn map_dxgi(value: u8) -> DxgiFormat {
    match value {
        0x00 | 0x01 => DxgiFormat::BC1_UNorm,
        0x02 => DxgiFormat::BC2_UNorm,
        0x03 => DxgiFormat::BC3_UNorm,
        0x06 => DxgiFormat::BC4_UNorm,
        0x07 => DxgiFormat::BC5_UNorm,
        0x22 => DxgiFormat::BC7_UNorm,
        0x23 => DxgiFormat::BC6H_UF16,
        _ => DxgiFormat::Unknown,
    }
}

fn map_fourcc(value: u8) -> FourCC {
    match value {
        0x00 | 0x01 => FourCC(FourCC::DXT1),
        0x02 => FourCC(FourCC::DXT3),
        0x03 => FourCC(FourCC::DXT5),
        0x06 => FourCC(FourCC::ATI1),
        0x07 => FourCC(FourCC::ATI2),
        _ => FourCC(FourCC::NONE),
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Image {
    pub header: DDSHeader,

    #[derivative(Debug = "ignore")]
    pub f_cache_image_count: u8,

    #[derivative(Debug = "ignore")]
    pub f_cache_image_offsets: Vec<u32>,
}

impl Image {
    pub fn from_with_header<T: Into<Vec<u8>>>(data: T, header: Header) -> Self {
        let data = data.into();
        let data = data[header.size..].to_vec();
        let mut data_offset = 0;

        // Skip unknown byte
        data_offset += 1;

        // Read the image count
        let f_cache_image_count: u8 = data[data_offset];
        data_offset += 1;

        // Skip unknown byte
        data_offset += 1;

        // Read the DDS compression format
        let fourcc = map_fourcc(data[data_offset]);
        let format = map_dxgi(data[data_offset]);
        data_offset += 1;

        // Read the mip map count
        let mip_map_count = u32::from_le_bytes([
            data[data_offset],
            data[data_offset + 1],
            data[data_offset + 2],
            data[data_offset + 3],
        ]);
        data_offset += 4;

        // Read the F cache image offsets
        let mut f_cache_image_offsets = Vec::new();
        for _ in 0..mip_map_count {
            f_cache_image_offsets.push(u32::from_le_bytes([
                data[data_offset],
                data[data_offset + 1],
                data[data_offset + 2],
                data[data_offset + 3],
            ]));
            data_offset += 4;
        }

        // Read the width ratio
        let width_ratio = u16::from_le_bytes([data[data_offset], data[data_offset + 1]]);
        data_offset += 2;

        // Read the height ratio
        let height_ratio = u16::from_le_bytes([data[data_offset], data[data_offset + 1]]);
        data_offset += 2;

        // Skip the B cache max width
        data_offset += 2;

        // Skip the B cache max height
        data_offset += 2;

        // Read the max side length
        let max_side_length = u32::from_le_bytes([
            data[data_offset],
            data[data_offset + 1],
            data[data_offset + 2],
            data[data_offset + 3],
        ]);
        // data_offset += 4;

        // Calculate the width and height
        let width: u32;
        let height: u32;
        if width_ratio > height_ratio {
            width = max_side_length;
            height = max_side_length * height_ratio as u32 / width_ratio as u32;
        } else {
            width = max_side_length * width_ratio as u32 / height_ratio as u32;
            height = max_side_length;
        }

        if fourcc != FourCC(FourCC::NONE) {
            let mut header = DDSHeader::default();
            header.width = width;
            header.height = height;
            header.spf.fourcc = Some(fourcc);

            return Image::new(header, f_cache_image_count, f_cache_image_offsets);
        } else {
            let header =
                DDSHeader::new_dxgi(height, width, None, format, None, None, None).unwrap();

            return Image::new(header, f_cache_image_count, f_cache_image_offsets);
        }
    }

    pub fn size(&self) -> usize {
        max(1, (self.header.width + 3) as usize / 4 as usize)
            * max(1, (self.header.height + 3) as usize / 4 as usize)
            * match self.header.spf.fourcc.clone().unwrap().0 {
                FourCC::DXT1 | FourCC::ATI1 => 8,
                FourCC::DXT3 | FourCC::DXT5 | FourCC::ATI2 | FourCC::DX10 => 16,
                _ => 8,
            }
    }
}

impl Image {
    pub fn new(
        header: DDSHeader,
        f_cache_image_count: u8,
        f_cache_image_offsets: Vec<u32>,
    ) -> Self {
        Self {
            header,
            f_cache_image_count,
            f_cache_image_offsets,
        }
    }
}

impl<T: Into<Vec<u8>>> From<T> for Image {
    fn from(data: T) -> Self {
        let data = data.into();
        let header = Header::from(data.clone());
        Self::from_with_header(data, header)
    }
}
