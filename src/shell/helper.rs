use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::line_buffer::LineBuffer;
use rustyline::validate::Validator;
use rustyline::{Context, Helper as RustylineHelper};

use crate::shell::command::ls::get_children;
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

impl RustylineHelper for Helper<'_> {}
