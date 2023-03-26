mod args;
mod audio;
mod metadata;
mod shell;
mod texture;
mod utils;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use log::{error, info};
use lotus_lib::cache_pair::{CachePair, CachePairReader};
use lotus_lib::package::{PackageCollection, PackageTrioType};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Editor};

use crate::shell::command::{cd, find, get, ls, pwd, stat};
use crate::shell::helper::Helper;
use crate::shell::State;

/// The enum of sub-commands supported by the CLI
#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Change the current working directory
    #[clap(name = "cd")]
    ChangeDirectory(cd::Arguments),

    /// Find a file or directory
    #[clap(name = "find")]
    FindFileOrDirectory(find::Arguments),

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
    let state = State::new(
        args.directory,
        args.output,
        h_cache.unwrap(),
        f_cache,
        b_cache,
    );
    let state = Rc::new(RefCell::new(state));

    // If the user specified a command, execute it and exit
    if let Some(command) = args.command {
        parse_command(&mut state.borrow_mut(), command.as_str())?;
        return Ok(());
    }

    let mut rl = Editor::<Helper>::new()?;
    rl.set_completion_type(CompletionType::List);
    rl.set_helper(Some(Helper {
        state: state.clone(),
    }));

    loop {
        let line = rl.readline(&format!(
            "[{} {}]# ",
            env!("CARGO_PKG_NAME"),
            state
                .borrow()
                .current_lotus_dir
                .file_name()
                .unwrap_or("/".as_ref())
                .to_str()
                .unwrap()
        ));
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

        // Parse the command
        parse_command(&mut state.borrow_mut(), line)?;
    }

    Ok(())
}

fn parse_command(state: &mut State, line: &str) -> Result<()> {
    // Trim the line
    let line: &str = line.trim();

    let mut command_parts = line.split(' ').collect::<Vec<&str>>();
    command_parts.insert(0, env!("CARGO_PKG_NAME"));

    match Cli::try_parse_from(command_parts.into_iter()) {
        Ok(command) => {
            let res = match command.command {
                Command::ChangeDirectory(args) => cd::command(state, args),
                Command::FindFileOrDirectory(args) => find::command(state, args),
                Command::GetFileContent(args) => get::command(state, args),
                Command::ListDirectoryContents(args) => ls::command(state, args),
                Command::PrintFileMetadata(args) => stat::command(state, args),
                Command::PrintWorkingDirectory(args) => pwd::command(state, args),
            };

            // Pad the output
            println!();

            return res;
        }
        Err(err) => Ok(match clap::Error::kind(&err) {
            ErrorKind::DisplayHelp => println!("{err}"),
            ErrorKind::DisplayVersion => println!("{err}"),
            _ => println!("Invalid command (type 'help' for help)\n"),
        }),
    }
}
