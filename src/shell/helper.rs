use std::cell::RefCell;
use std::rc::Rc;

use lotus_lib::toc::node::Node;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::line_buffer::LineBuffer;
use rustyline::validate::Validator;
use rustyline::{Context, Helper as RustylineHelper};

use crate::shell::State;

pub struct Helper<'a> {
    pub state: Rc<RefCell<State<'a>>>,
}

impl Validator for Helper<'_> {}

impl Highlighter for Helper<'_> {}

impl Hinter for Helper<'_> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        let _ = (line, pos, ctx);
        None
    }
}

impl Completer for Helper<'_> {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut candidates: Vec<String> = Vec::new();

        // Get the command
        let command = line.split_whitespace().next();

        // If there is no command, return an empty list
        if command.is_none() || command.unwrap().len() == line.len() {
            return Ok((0, Vec::with_capacity(0)));
        }

        // Get the last argument of the command
        let last_arg_pos = line.rfind(' ').unwrap_or(pos) + 1;
        let last_arg = &line[last_arg_pos..pos];

        // If the last argument contains a slash, split it and get the last part
        let uncompleted = last_arg.split('/').last().unwrap_or(last_arg);
        let arg_path = &last_arg[..last_arg.len() - uncompleted.len()];

        // Get the current directory
        let mut current_dir = self.state.borrow().current_lotus_dir.clone();
        current_dir.push(arg_path);

        // Get the current directory node
        let current_dir_node = self
            .state
            .borrow()
            .h_cache
            .get_directory_node(current_dir.to_str().unwrap());

        // Check if the directory exists
        if current_dir_node.is_none() {
            return Ok((0, Vec::with_capacity(0)));
        }

        // Get the directory node
        let current_dir_node = current_dir_node.unwrap();

        // Get matching directories
        for child_directory in current_dir_node.borrow().children_directories() {
            if child_directory.borrow().name().starts_with(uncompleted) {
                candidates.push(format!(
                    "{}/",
                    &child_directory.borrow().path().display().to_string()
                        [current_dir.display().to_string().len()..]
                ));
            }
        }

        // Get matching files
        for child_file in current_dir_node.borrow().children_files() {
            if child_file.borrow().name().starts_with(uncompleted) {
                candidates.push(
                    child_file.borrow().path().display().to_string()
                        [current_dir.display().to_string().len()..].to_string(),
                );
            }
        }

        // Sort the candidates
        candidates.sort();
        
        Ok((last_arg_pos + arg_path.len(), candidates))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected);
    }
}

impl RustylineHelper for Helper<'_> {}
