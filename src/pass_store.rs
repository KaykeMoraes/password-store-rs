use anyhow::{Result, anyhow, ensure};
use std::{
    env::{self, home_dir},
    fs::{File, create_dir, read, remove_file},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

use crate::{decrypt, encrypt};

pub struct PassStore {
    store_path: PathBuf,
    clipboard: bool,
}

impl PassStore {
    pub fn new(store_path: &Path, clipboard: bool) -> Self {
        Self {
            store_path: PathBuf::from(store_path),
            clipboard,
        }
    }

    pub fn get_password_path(&self, pass_name: &str) -> PathBuf {
        self.store_path.join(pass_name).with_added_extension("age")
    }

    pub fn clipoard(self, yes: bool) -> Self {
        Self::new(&self.store_path, yes)
    }

    pub fn store_path(self, path: &Path) -> Self {
        Self::new(path, self.clipboard)
    }

    pub fn get_store_path(&self) -> &Path {
        self.store_path.as_path()
    }

    pub fn init(&self) -> Result<()> {
        if self.store_path.try_exists()? && !self.store_path.is_empty() {
            Err(anyhow!(
                "The provided path {0} already exists and is not empty",
                self.store_path.display()
            ))
        } else if self.store_path.is_file() {
            Err(anyhow!(
                "The provided path {0} points to a file",
                self.store_path.display()
            ))
        } else {
            create_dir(self.store_path.as_path())?;
            Ok(())
        }
    }

    pub fn list(path: &Path) -> Result<()> {
        let dir = path.read_dir()?;

        for entry in dir {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                Self::list(&entry.path())?;
            } else {
                println!("{}", entry.path().display())
            }
        }

        Ok(())
    }

    pub fn insert(&self, pass_name: &str, password: &str, passphrase: &str) -> Result<()> {
        let password_path = self.get_password_path(pass_name);

        if !password_path.try_exists()? {
            let encrypted_pass = encrypt(password.as_bytes(), passphrase)?;

            let mut password_file = File::create_new(password_path.as_path())?;

            password_file.write_all(encrypted_pass.as_slice())?;

            Ok(())
        } else {
            Err(anyhow!(
                "A file with the same name already exists at {0}",
                password_path.display()
            ))
        }
    }

    pub fn show(&self, pass_name: &str, passphrase: &str) -> Result<()> {
        let password_path = self.get_password_path(pass_name);

        let file_contents = read(password_path)?;

        let password = String::from_utf8(decrypt(file_contents.as_slice(), passphrase)?)?;

        if self.clipboard {
            send_to_clipboard(&password)
        } else {
            println!("{}", password);
            Ok(())
        }
    }

    pub fn edit(&self, pass_name: &str, passphrase: &str) -> Result<()> {
        let editor = if let Ok(editor) = env::var("EDITOR") {
            editor
        } else if let Ok(editor) = env::var("VISUAL") {
            editor
        } else {
            String::from("nano")
        };

        let password_path = self.get_password_path(pass_name);

        if password_path.try_exists()? {
            let mut temp_file = NamedTempFile::new()?;

            let password = decrypt(read(password_path.as_path())?.as_slice(), passphrase)?;

            temp_file.write_all(password.as_slice())?;

            let status = Command::new(editor).arg(temp_file.path()).status()?;

            ensure!(status.success());

            let edited_password = read(temp_file.path())?;

            let mut pass_file = File::create(password_path.as_path())?;

            pass_file.write_all(edited_password.as_slice())?;

            Ok(())
        } else {
            Err(anyhow!(
                "The password {pass_name} was not found in the store"
            ))
        }
    }

    pub fn remove(&self, pass_name: &str) -> Result<()> {
        let password_path = self.get_password_path(pass_name);

        remove_file(&password_path)?;

        Ok(())
    }
}

impl Default for PassStore {
    fn default() -> Self {
        let store_path = home_dir().unwrap().join(".password-store");

        Self::new(&store_path, false)
    }
}

fn send_to_clipboard(content: &str) -> Result<()> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wl-copy").arg(content).status()?;
        Ok(())
    } else {
        let echo = Command::new("echo")
            .arg(content)
            .stdout(Stdio::piped())
            .spawn()?;

        Command::new("xclip")
            .arg("-i")
            .stdin(Stdio::from(echo.stdout.unwrap()))
            .status()?;
        Ok(())
    }
}

mod test {
    use super::*;

    #[test]
    fn test_clipboard() -> Result<()> {
        let content = "Hello";

        if env::var("WAYLAND_DISPLAY").is_ok() {
            send_to_clipboard(content)?;

            let clipboard_content = Command::new("wl-paste").output()?;

            assert!(clipboard_content.status.success());

            assert_eq!(clipboard_content.stdout.trim_ascii(), content.as_bytes());

            Ok(())
        } else {
            send_to_clipboard(content)?;

            let clipboard_content = Command::new("xclip").arg("-o").output()?;

            assert!(clipboard_content.status.success());

            assert_eq!(clipboard_content.stdout.trim_ascii(), content.as_bytes());

            Ok(())
        }
    }
}
