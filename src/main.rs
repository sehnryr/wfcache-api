mod args;
mod shell;
mod utils;

use clap::Parser;
use log::{error, info};
use lotus_lib::{
    cache_pair::{CachePair, CachePairReader},
    package::{PackageCollection, PackageTrioType},
};
use shell::command::{cd, ls, pwd, stat};
use shellfish::{clap_command, Shell};

use crate::shell::State;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Parse arguments
    let args = args::Args::parse();

    // Initialize the package collection
    let collection = PackageCollection::<CachePairReader>::new(args.directory.clone(), true);
    info!(
        "Loaded {} packages: {:?}",
        collection.packages().len(),
        collection.packages().keys().collect::<Vec<_>>()
    );

    // Get the package
    let package = collection.get_package(args.package.as_str());
    if package.is_none() {
        error!("Package not found: {}", args.package);
        std::process::exit(1);
    }
    let package = package.unwrap();

    // Get the header
    let header = package.get(&PackageTrioType::H);
    if header.is_none() {
        std::process::exit(1);
    }
    let header = header.unwrap();

    // Load the header
    header.read_toc();

    // Initialize the state
    let state = State::new(args.directory, args.package, header);

    // Define a shell
    let mut shell = Shell::new(state, "wfcache-api$ ");

    // Add ls command
    shell
        .commands
        .insert("ls", clap_command!(State, ls::Arguments, ls::command));

    // Add cd command
    shell
        .commands
        .insert("cd", clap_command!(State, cd::Arguments, cd::command));

    // Add pwd command
    shell
        .commands
        .insert("pwd", clap_command!(State, pwd::Arguments, pwd::command));

    // Add stat command
    shell
        .commands
        .insert("stat", clap_command!(State, stat::Arguments, stat::command));

    // Run the shell
    shell.run()?;

    Ok(())
}
