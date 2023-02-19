#[derive(Debug)]
pub struct PathNotFound;

impl std::fmt::Display for PathNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Path not found")
    }
}

impl std::error::Error for PathNotFound {}

#[derive(Debug)]
pub struct MissingArgument;

impl std::fmt::Display for MissingArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing argument")
    }
}

impl std::error::Error for MissingArgument {}