use lotus_lib::cache_pair::CachePairReader;

pub struct State<'a> {
    pub directory: std::path::PathBuf,
    pub package: String,
    pub cache: &'a CachePairReader,
    pub current_lotus_dir: std::path::PathBuf,
}

impl State<'_> {
    pub fn new(directory: std::path::PathBuf, package: String, cache: &CachePairReader) -> State {
        State {
            directory,
            package,
            cache,
            current_lotus_dir: std::path::PathBuf::from("/"),
        }
    }
}
