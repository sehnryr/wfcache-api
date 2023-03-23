mod extract;
mod header;
mod ogg;
mod opus;

pub use extract::extract;

/// The file type of a music file
pub struct AudioType(pub u32);

impl AudioType {
    pub const AUDIO_139: u32 = 0x8B; // TODO: Rename to something more descriptive
}
