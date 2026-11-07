use std::{env::home_dir, path::PathBuf};

use clap::Parser;
use pass::{
    cli::{AppCommand, Cli},
    pass_store,
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let store_path = if let Some(path) = cli.path {
        PathBuf::from(path)
    } else {
        home_dir().unwrap().join(".pass-store")
    };

    match cli.command {
        AppCommand::Edit { pass_name } => {
            let passphrase = rpassword::prompt_password("Type your passphrase: ")?;

            pass_store::edit(&store_path, &pass_name, &passphrase)
        }
        AppCommand::Find { pass_names } => {
            pass_store::find_password(&store_path, &pass_names);
            Ok(())
        }
        AppCommand::Init => pass_store::init(&store_path),
        AppCommand::Insert { echo, pass_name } => {
            let passphrase = rpassword::prompt_password("Type your passphrase: ")?;

            let password =
                rpassword::prompt_password(format!("Type the password for {pass_name}: "))?;

            let password_confirmation =
                rpassword::prompt_password(format!("Retype the password for {pass_name}: "))?;

            anyhow::ensure!(password == password_confirmation, "The ");

            pass_store::insert(
                &store_path,
                &pass_name,
                &password,
                &passphrase,
                echo.unwrap_or(false),
            )
        }
        AppCommand::List { subfolder } => {
            if let Some(subfolder) = subfolder {
                pass_store::list(&store_path.join(subfolder))
            } else {
                pass_store::list(&store_path)
            }
        }
        AppCommand::Remove { pass_name } => pass_store::remove(&store_path, &pass_name),
        AppCommand::Show {
            clipboard,
            pass_name,
        } => {
            let passphrase = rpassword::prompt_password("Type your passphrase: ")?;

            pass_store::show(&store_path, &pass_name, &passphrase, clipboard)
        }
    }
}
