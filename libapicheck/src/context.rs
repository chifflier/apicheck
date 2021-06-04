use crate::config::Config;
use rustc_ast_pretty::pprust::state::State;

pub struct Context<'a> {
    pub config: &'a Config,

    state: State<'a>,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a Config) -> Self {
        Context {
            config,
            state: State::new(),
        }
    }

    pub fn print_state(&self) -> &'a State {
        &self.state
    }
}
