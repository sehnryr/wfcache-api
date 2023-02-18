#[derive(Debug)]
pub struct PathNotFound;

impl std::fmt::Display for PathNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Path not found")
    }
}

impl std::error::Error for PathNotFound {}
