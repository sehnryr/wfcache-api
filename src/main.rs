mod args;
mod metadata;
mod music;
mod shell;
mod texture;
mod utils;

use anyhow::Result;
use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use log::{error, info};
use lotus_lib::cache_pair::{CachePair, CachePairReader};
use lotus_lib::package::{PackageCollection, PackageTrioType};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::shell::command::{cd, get, ls, pwd, stat};
use crate::shell::State;

/// The enum of sub-commands supported by the CLI
#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Change the current working directory
    #[clap(name = "cd")]
    ChangeDirectory(cd::Arguments),

    /// Get the contents of a file
    #[clap(name = "get")]
    GetFileContent(get::Arguments),

    /// List the contents of a directory
    #[clap(name = "ls")]
    ListDirectoryContents(ls::Arguments),

    /// Print the metadata of a file
    #[clap(name = "stat")]
    PrintFileMetadata(stat::Arguments),

    /// Print the current working directory
    #[clap(name = "pwd")]
    PrintWorkingDirectory(pwd::Arguments),
}

#[derive(Parser, Clone, Debug)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
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
    let mut state = State::new(args.directory, h_cache.unwrap(), f_cache, b_cache);

    let mut rl = Editor::<()>::new()?;
    let prompt = "wfcache-api$ ";

    loop {
        let line = rl.readline(&prompt);
        match line {
            Ok(_) => {}
            Err(ReadlineError::Interrupted) => break, // Ctrl-C
            Err(ReadlineError::Eof) => break,         // Ctrl-D
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }

        // Trim the line
        let line: String = line.unwrap();
        let line: &str = line.trim();

        // Get the command if it exists
        let parts: Vec<&str> = line.split(' ').collect();
        let mut command = String::new();
        if let Some(head) = parts.first() {
            command = String::from(*head);
        }

        // Handle quit/exit commands and skip if empty
        match command.to_lowercase().as_str() {
            "" => continue, // Do nothing
            "quit" | "exit" => break,
            _ => {}
        }

        // Add the line to the history
        rl.add_history_entry(line);

        let mut command_parts = parts;
        command_parts.insert(0, env!("CARGO_PKG_NAME"));

        match match Cli::try_parse_from(command_parts.into_iter()) {
            Ok(command) => match command.command {
                Command::ChangeDirectory(args) => cd::command(&mut state, args),
                Command::GetFileContent(args) => get::command(&mut state, args),
                Command::ListDirectoryContents(args) => ls::command(&mut state, args),
                Command::PrintFileMetadata(args) => stat::command(&mut state, args),
                Command::PrintWorkingDirectory(args) => pwd::command(&mut state, args),
            },
            Err(err) => Ok(match clap::Error::kind(&err) {
                ErrorKind::DisplayHelp => println!("{err}"),
                ErrorKind::DisplayVersion => println!("{err}"),
                _ => println!("Invalid command (type 'help' for help)"),
            }),
        } {
            Ok(_) => {}
            Err(err) => println!("{err}"),
        }

        // Pad the output
        println!();
    }

    Ok(())
}
