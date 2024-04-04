mod action;
mod app;
mod args;
mod errors;
mod extract;
mod tui;
mod widgets;

use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    errors::install_hooks()?;
    let args = args::Args::parse();

    // Initialize the ratatui terminal
    let mut tui = tui::Tui::new()
        .wrap_err("Failed to initialize TUI")?
        .frame_rate(30)
        .tick_rate(30);
    tui.enter().wrap_err("Failed to enter TUI")?;

    // Run the ratatui app
    app::App::try_init(args.directory, args.package, args.output)?
        .run(&mut tui)
        .await?;

    // Exit the ratatui terminal
    tui.exit().wrap_err("Failed to exit TUI")?;
    Ok(())
}
