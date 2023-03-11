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

    /// Execute a command without opening the shell
    #[arg(short, long)]
    pub command: Option<String>,
}
