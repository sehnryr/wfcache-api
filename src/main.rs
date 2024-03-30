mod app;
mod args;
mod errors;
mod tui;
mod widgets;

use clap::Parser;
use color_eyre::Result;

fn main() -> Result<()> {
    errors::install_hooks()?;
    let args = args::Args::parse();
    let mut terminal = tui::init()?;
    app::App::try_init(args.directory, args.package, args.output)?.run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}
