use crate::shell::State;
use clap::Parser;

/// Print the name of the current working directory
#[derive(Parser, Debug, Clone)]
pub struct Arguments {}

pub fn command(state: &State, _args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", state.current_lotus_dir.to_str().unwrap());

    Ok(())
}
