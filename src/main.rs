mod args;
mod shell;
mod utils;

use crate::shell::{
    command::{cd, get, ls, pwd, stat},
    Handler, State,
};
use clap::Parser;
use log::{error, info};
use lotus_lib::{
    cache_pair::{CachePair, CachePairReader},
    package::{PackageCollection, PackageTrioType},
};
use shellfish::{clap_command, Shell};

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

    let h_cache = package.get(&PackageTrioType::H);
    let f_cache = package.get(&PackageTrioType::F);
    let b_cache = package.get(&PackageTrioType::B);

    if h_cache.is_none() {
        std::process::exit(1);
    } else {
        h_cache.unwrap().read_toc().unwrap();
    }
    if f_cache.is_some() {
        f_cache.unwrap().read_toc().unwrap();
    }
    if b_cache.is_some() {
        b_cache.unwrap().read_toc().unwrap();
    }

    // Initialize the state
    let state = State::new(args.directory, h_cache.unwrap(), f_cache, b_cache);

    // Define a shell
    let mut shell = Shell::new_with_handler(state, "wfcache-api$ ", Handler());

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

    // Add get command
    shell
        .commands
        .insert("get", clap_command!(State, get::Arguments, get::command));

    // Run the shell
    shell.run()?;

    Ok(())
}
