mod app;
mod args;
mod errors;
mod input;
mod tui;
mod widgets;

use clap::Parser;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::package::PackageCollection;

fn main() -> Result<()> {
    errors::install_hooks()?;
    let args = args::Args::parse();
    let collection = PackageCollection::<CachePairReader>::new(args.directory, true);
    let package = collection
        .get_package(&args.package)
        .wrap_err(format!("Package {} not found", &args.package))?;
    let mut terminal = tui::init()?;
    app::App::try_init(package, args.output)?.run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}
