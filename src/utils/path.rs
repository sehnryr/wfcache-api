use relative_path::RelativePathBuf;
use std::path::PathBuf;

pub fn normalize_path(path: &PathBuf, base: &PathBuf) -> PathBuf {
    let mut directory = path.clone();

    // Normalize the path
    if !directory.is_absolute() {
        // Wtf
        directory = base.join(directory);
        directory = RelativePathBuf::from(directory.to_str().unwrap())
            .normalize()
            .to_path("");
        directory = PathBuf::from("/").join(directory);
    }

    directory
}