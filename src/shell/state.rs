use lotus_lib::cache_pair::CachePairReader;

pub struct State<'a> {
    pub directory: std::path::PathBuf,
    pub h_cache: &'a CachePairReader,
    pub f_cache: Option<&'a CachePairReader>,
    pub b_cache: Option<&'a CachePairReader>,
    pub current_lotus_dir: std::path::PathBuf,
}

impl State<'_> {
    pub fn new<'a>(
        directory: std::path::PathBuf,
        h_cache: &'a CachePairReader,
        f_cache: Option<&'a CachePairReader>,
        b_cache: Option<&'a CachePairReader>,
    ) -> State<'a> {
        State {
            directory,
            h_cache,
            f_cache,
            b_cache,
            current_lotus_dir: std::path::PathBuf::from("/"),
        }
    }
}
