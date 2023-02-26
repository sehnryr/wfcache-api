use anyhow::Result;
use clap::Parser;

use crate::shell::State;

/// Print the name of the current working directory
#[derive(Parser, Debug, Clone)]
pub struct Arguments {}

pub fn command(state: &State, _args: Arguments) -> Result<()> {
    println!("{}", state.current_lotus_dir.to_str().unwrap());

    Ok(())
}
