mod action;
mod app;
mod args;
mod errors;
mod tui;
mod widgets;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use lotus_lib::cache_pair::CachePairReader;
use lotus_lib::package::PackageCollection;

#[tokio::main]
async fn main() -> Result<()> {
    errors::install_hooks()?;
    let args = args::Args::parse();
    let collection = PackageCollection::<CachePairReader>::new(args.directory, true);
    let package = collection
        .get_package(&args.package)
        .wrap_err(format!("Package {} not found", &args.package))?;

    // Initialize the ratatui terminal
    let mut tui = tui::Tui::new()
        .wrap_err("Failed to initialize TUI")?
        .frame_rate(30)
        .tick_rate(1);
    tui.enter().wrap_err("Failed to enter TUI")?;

    // Run the ratatui app
    app::App::try_init(package, args.output)?
        .run(&mut tui)
        .await?;

    // Exit the ratatui terminal
    tui.exit().wrap_err("Failed to exit TUI")?;
    Ok(())
}
