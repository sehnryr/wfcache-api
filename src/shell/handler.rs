use std::collections::HashMap;
use yansi::Paint;

/// This handler replaces the default handler in the shellfish crate.
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct Handler();

impl<T> shellfish::Handler<T> for Handler {
    fn handle(
        &self,
        line: Vec<String>,
        commands: &HashMap<&str, shellfish::Command<T>>,
        state: &mut T,
        _description: &str,
    ) -> bool {
        if let Some(command) = line.get(0) {
            match command.as_str() {
                "quit" | "exit" => return true,
                "help" => {
                    const NAME: &str = env!("CARGO_PKG_NAME");
                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
                    println!("{}, version {}", NAME, VERSION);
                    println!("{}", DESCRIPTION);
                    println!();

                    // Sort commands by name
                    let mut sorted_commands: Vec<_> = commands.iter().collect();
                    sorted_commands.sort_by(|a, b| a.0.cmp(b.0));

                    // Print information about custom commands
                    for (name, command) in sorted_commands {
                        println!(" {} \t{}", name, command.help);
                    }

                    // Print information about built-in commands
                    println!(" help \tdisplays help information");
                    println!(" quit \tquits the shell");
                    println!(" exit \texits the shell");
                }
                _ => {
                    // Attempt to find the command
                    let command = commands.get(&*line[0]);

                    // Checks if we got it
                    match command {
                        Some(command) => {
                            if let Err(e) = match command.command {
                                shellfish::command::CommandType::Sync(c) => c(state, line),
                            } {
                                eprintln!(
                                    "{}",
                                    Paint::red(format!(
                                        "Command exited unsuccessfully:\n{}\n({:?})",
                                        &e, &e
                                    ))
                                )
                            }
                        }
                        None => {
                            eprintln!("{} {}", Paint::red("Command not found:"), line[0])
                        }
                    }
                }
            }

            // Padding
            println!();
        }
        false
    }
}
