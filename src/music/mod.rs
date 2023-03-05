mod extract;

pub use extract::extract;

/// The file type of a music file
pub struct MusicType(pub u32);

impl MusicType {
    pub const MUSIC_139: u32 = 0x8B; // TODO: Rename to something more descriptive
}
