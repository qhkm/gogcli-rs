use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Generate shell completion scripts.
#[derive(clap::Args, Debug)]
pub struct CompletionCmd {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

impl CompletionCmd {
    pub fn execute(&self) {
        let mut cmd = crate::Cli::command();
        generate(self.shell, &mut cmd, "gog", &mut io::stdout());
    }
}
