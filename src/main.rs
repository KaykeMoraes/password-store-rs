use clap::Parser;
use pass::cli::{self, AppCommand, Cli};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        AppCommand::Edit { pass_name } => {}
        AppCommand::Find { pass_names } => {}
        AppCommand::Init => {}
        AppCommand::Insert { echo, pass_name } => {}
        AppCommand::List { subfolder } => {}
        AppCommand::Remove { pass_name } => {}
        AppCommand::Show {
            clipboard,
            pass_name,
        } => {}
    }
}
