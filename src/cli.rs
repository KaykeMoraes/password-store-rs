use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "pass", about = "A simple unix password manager", version)]
pub struct Cli {
    #[arg(long = "path", short = 'p')]
    pub path: Option<String>,
    #[command(subcommand)]
    pub command: AppCommand,
}

#[derive(Debug, Subcommand)]
pub enum AppCommand {
    #[command(name = "init", about = "Initialize a new password storage")]
    Init,
    #[command(name = "list", alias = "ls", about = "List passwords in the store")]
    List {
        #[arg(id = "subfolder")]
        subfolder: Option<String>,
    },
    #[command(
        name = "insert",
        about = "Insert a new password and optionally print it back to the console"
    )]
    Insert {
        #[arg(id = "echo", short = 'e')]
        echo: Option<bool>,
        #[arg(id = "pass-name", required = true)]
        pass_name: String,
    },
    #[command(
        name = "show",
        about = "Show an existing password and optionally send it to the clipboard"
    )]
    Show {
        #[arg(id = "clipboard", short = 'c')]
        clipboard: bool,
        #[arg(id = "pass-name", required = true)]
        pass_name: String,
    },
    #[command(name = "edit", about = "Edit an existing password")]
    Edit {
        #[arg(id = "pass-name", required = true)]
        pass_name: String,
    },
    #[command(name = "remove", alias = "rm", about = "Remove a password")]
    Remove {
        #[arg(id = "pass-name", required = true)]
        pass_name: String,
    },
    #[command(name = "find", about = "Find passwords that matches the given names")]
    Find {
        #[arg(id = "pass-names", required = true)]
        pass_names: Vec<String>,
    },
}
