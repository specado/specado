//! Shell completions command handler

use crate::cli::CompletionsArgs;
use crate::error::Result;
use clap::CommandFactory;

/// Handle the completions command
pub fn handle_completions(args: CompletionsArgs) -> Result<()> {
    use clap_complete::generate;
    use std::io;
    
    let mut cmd = crate::cli::Cli::command();
    let name = cmd.get_name().to_string();
    
    generate(
        args.shell.to_clap_shell(),
        &mut cmd,
        name,
        &mut io::stdout(),
    );
    
    Ok(())
}