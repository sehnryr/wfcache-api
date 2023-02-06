use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to use
    #[arg(short, long)]
    pub directory: std::path::PathBuf,

    /// Package to search for
    #[arg(short, long, default_value = "Misc")]
    pub package: String,

    /// Lotus path to search for
    #[arg(short, long, default_value = "/")]
    pub lotus_path: String,

    /// List the content of a directory
    #[arg(long, conflicts_with = "extract")]
    pub ls: bool,

    /// Extract a file or directory to the current directory (not implemented)
    #[arg(long, conflicts_with = "ls")]
    pub extract: Option<String>,

    /// Overwrite existing files when extracting (not implemented)
    #[arg(long, conflicts_with = "ls")]
    pub overwrite: bool,
}
