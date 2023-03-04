mod extract;
mod header;

pub use extract::extract;

/// The file type of a texture
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TextureType(pub u32);

impl TextureType {
    pub const DIFFUSE_EMISSION_TINT: u32 = 0xA3;
    pub const BILLBOARD_SPRITEMAP_DIFFUSE: u32 = 0xA4;
    pub const BILLBOARD_SPRITEMAP_NORMAL: u32 = 0xA5;
    pub const ROUGHNESS: u32 = 0xA7;
    pub const SKYBOX: u32 = 0xAB;
    pub const TEXTURE_174: u32 = 0xAE; // TODO: Rename to something more descriptive
    pub const TEXTURE_176: u32 = 0xB0; // TODO: Rename to something more descriptive
    pub const CUBEMAP: u32 = 0xB1;
    pub const NORMAL_MAP: u32 = 0xB8;
    pub const PACKMAP: u32 = 0xBC;
    pub const TEXTURE_194: u32 = 0xC2; // TODO: Rename to something more descriptive
    pub const DETAILSPACK: u32 = 0xC3;
}
