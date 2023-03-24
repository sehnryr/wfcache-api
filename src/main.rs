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
use rustyline::completion::Completer;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::line_buffer::LineBuffer;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Context, Editor, Helper};

use crate::shell::command::ls::get_children;
use crate::shell::command::{cd, find, get, ls, pwd, stat};
use crate::shell::State;

struct ApiHelper<'a> {
    state: Rc<RefCell<State<'a>>>,
}

impl Validator for ApiHelper<'_> {}

impl Highlighter for ApiHelper<'_> {}

impl Hinter for ApiHelper<'_> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        let _ = (line, pos, ctx);
        None
    }
}

impl Completer for ApiHelper<'_> {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut candidates = Vec::new();

        // Get the command
        let command = line.split_whitespace().next();

        // If there is no command, return an empty list
        if command.is_none() || command.unwrap().len() == line.len() {
            return Ok((0, Vec::with_capacity(0)));
        }

        // Get the last argument of the command
        let last_arg_pos = line.trim_end().rfind(' ').unwrap_or(pos) + 1;
        let last_arg = &line[last_arg_pos..pos];

        // Get the current directory
        let current_dir = self.state.borrow().current_lotus_dir.clone();

        // Get the current directory node
        let current_dir_node = self
            .state
            .borrow()
            .h_cache
            .get_directory_node(current_dir.to_str().unwrap())
            .unwrap();

        for node in get_children(current_dir_node) {
            if node.1.starts_with(last_arg) {
                candidates.push(node.1);
            }
        }

        Ok((last_arg_pos, candidates))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected);
    }
}

impl Helper for ApiHelper<'_> {}

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
    let state = State::new(args.directory, h_cache.unwrap(), f_cache, b_cache);
    let state = Rc::new(RefCell::new(state));

    // If the user specified a command, execute it and exit
    if let Some(command) = args.command {
        parse_command(&mut state.borrow_mut(), command.as_str())?;
        return Ok(());
    }

    let mut rl = Editor::<ApiHelper>::new()?;
    rl.set_completion_type(CompletionType::List);
    rl.set_helper(Some(ApiHelper {
        state: state.clone(),
    }));
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

        // Parse the command
        parse_command(&mut state.borrow_mut(), line)?;

        // Pad the output
        println!();
    }

    Ok(())
}

fn parse_command(state: &mut State, line: &str) -> Result<()> {
    // Trim the line
    let line: &str = line.trim();

    let mut command_parts = line.split(' ').collect::<Vec<&str>>();
    command_parts.insert(0, env!("CARGO_PKG_NAME"));

    match Cli::try_parse_from(command_parts.into_iter()) {
        Ok(command) => match command.command {
            Command::ChangeDirectory(args) => cd::command(state, args),
            Command::FindFileOrDirectory(args) => find::command(state, args),
            Command::GetFileContent(args) => get::command(state, args),
            Command::ListDirectoryContents(args) => ls::command(state, args),
            Command::PrintFileMetadata(args) => stat::command(state, args),
            Command::PrintWorkingDirectory(args) => pwd::command(state, args),
        },
        Err(err) => Ok(match clap::Error::kind(&err) {
            ErrorKind::DisplayHelp => println!("{err}"),
            ErrorKind::DisplayVersion => println!("{err}"),
            _ => println!("Invalid command (type 'help' for help)"),
        }),
    }
}
