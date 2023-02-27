use anyhow::{Error, Result};
use ddsfile::{DxgiFormat, FourCC, Header as DDSHeader, PixelFormatFlags};
use derivative::Derivative;
use std::cmp::max;

use crate::utils::header::Header;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DDSFormat {
    BC1_UNORM,
    BC2_UNORM,
    BC3_UNORM,
    BC4_UNORM,
    BC5_UNORM,
    BC6H_UF16,
    BC7_UNORM,
    Uncompressed,
    Unknown,
}

impl DDSFormat {
    fn bits_per_pixel(&self) -> u32 {
        match self {
            DDSFormat::BC1_UNORM => 8,
            DDSFormat::BC2_UNORM => 16,
            DDSFormat::BC3_UNORM => 16,
            DDSFormat::BC4_UNORM => 8,
            DDSFormat::BC5_UNORM => 16,
            DDSFormat::BC6H_UF16 => 16,
            DDSFormat::BC7_UNORM => 16,
            DDSFormat::Uncompressed => 64,
            DDSFormat::Unknown => 0,
        }
    }

    fn to_dxgi_format(&self) -> DxgiFormat {
        match self {
            DDSFormat::BC1_UNORM => DxgiFormat::BC1_UNorm,
            DDSFormat::BC2_UNORM => DxgiFormat::BC2_UNorm,
            DDSFormat::BC3_UNORM => DxgiFormat::BC3_UNorm,
            DDSFormat::BC4_UNORM => DxgiFormat::BC4_UNorm,
            DDSFormat::BC5_UNORM => DxgiFormat::BC5_UNorm,
            DDSFormat::BC6H_UF16 => DxgiFormat::BC6H_UF16,
            DDSFormat::BC7_UNORM => DxgiFormat::BC7_UNorm,
            DDSFormat::Uncompressed => DxgiFormat::R8G8B8A8_UNorm,
            DDSFormat::Unknown => DxgiFormat::Unknown,
        }
    }

    fn to_fourcc(&self) -> Option<FourCC> {
        match self {
            DDSFormat::BC1_UNORM => Some(FourCC(FourCC::DXT1)),
            DDSFormat::BC2_UNORM => Some(FourCC(FourCC::DXT3)),
            DDSFormat::BC3_UNORM => Some(FourCC(FourCC::DXT5)),
            DDSFormat::BC4_UNORM => Some(FourCC(FourCC::ATI1)),
            DDSFormat::BC5_UNORM => Some(FourCC(FourCC::ATI2)),
            DDSFormat::BC6H_UF16 => Some(FourCC(FourCC::DX10)),
            DDSFormat::BC7_UNORM => Some(FourCC(FourCC::DX10)),
            DDSFormat::Uncompressed => None,
            DDSFormat::Unknown => None,
        }
    }
}

impl From<u8> for DDSFormat {
    fn from(value: u8) -> Self {
        match value {
            0x00 | 0x01 => DDSFormat::BC1_UNORM,
            0x02 => DDSFormat::BC2_UNORM,
            0x03 => DDSFormat::BC3_UNORM,
            0x06 => DDSFormat::BC4_UNORM,
            0x07 => DDSFormat::BC5_UNORM,
            0x22 => DDSFormat::BC6H_UF16,
            0x23 => DDSFormat::BC7_UNORM,
            0x0A => DDSFormat::Uncompressed,
            _ => DDSFormat::Unknown,
        }
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

    size: usize,
}

impl Image {
    pub fn from_with_header<T: Into<Vec<u8>>>(data: T, header: Header) -> Result<Self> {
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
        let dds_format = DDSFormat::from(data[data_offset]);
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

        // Calculate the size
        let size: usize =
            (max(1, width >> 2) * max(1, height >> 2) * dds_format.bits_per_pixel()) as usize;

        if dds_format == DDSFormat::Uncompressed {
            let mut header = DDSHeader::default();
            header.width = width;
            header.height = height;
            header.pitch = Some(width * dds_format.bits_per_pixel() >> 3);

            header.spf.fourcc = None;
            header.spf.flags.insert(PixelFormatFlags::ALPHA_PIXELS);
            header.spf.flags.insert(PixelFormatFlags::RGB);
            header.spf.rgb_bit_count = Some(32);
            header.spf.r_bit_mask = Some(0x00FF0000);
            header.spf.g_bit_mask = Some(0x0000FF00);
            header.spf.b_bit_mask = Some(0x000000FF);
            header.spf.a_bit_mask = Some(0xFF000000);

            return Ok(Image::new(
                header,
                f_cache_image_count,
                f_cache_image_offsets,
                size,
            ));
        }

        let fourcc = dds_format.to_fourcc();
        if fourcc != Some(FourCC(FourCC::NONE)) {
            let mut header = DDSHeader::default();
            header.width = width;
            header.height = height;
            header.spf.fourcc = fourcc;

            return Ok(Image::new(
                header,
                f_cache_image_count,
                f_cache_image_offsets,
                size,
            ));
        }

        let dxgi_format = dds_format.to_dxgi_format();
        if dxgi_format != DxgiFormat::Unknown {
            let header =
                DDSHeader::new_dxgi(height, width, None, dxgi_format, None, None, None).unwrap();

            return Ok(Image::new(
                header,
                f_cache_image_count,
                f_cache_image_offsets,
                size,
            ));
        }

        Err(Error::msg("Unknown DDS format"))
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Image {
    pub fn new(
        header: DDSHeader,
        f_cache_image_count: u8,
        f_cache_image_offsets: Vec<u32>,
        size: usize,
    ) -> Self {
        Self {
            header,
            f_cache_image_count,
            f_cache_image_offsets,
            size,
        }
    }
}
