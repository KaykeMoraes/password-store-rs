use anyhow::{Result, anyhow, ensure};
use std::{
    env::{self},
    fs::{File, create_dir, read, read_dir, remove_file},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

use crate::{decrypt, encrypt};

pub fn get_password_path(store_path: &Path, pass_name: &str) -> PathBuf {
    store_path.join(pass_name).with_added_extension("age")
}

pub fn init(store_path: &Path) -> Result<()> {
    if store_path.try_exists()? && store_path.is_empty() {
        Err(anyhow!(
            "The provided path {0} already exists and is not empty",
            store_path.display()
        ))
    } else if store_path.is_file() {
        Err(anyhow!(
            "The provided path {0} points to a file",
            store_path.display()
        ))
    } else {
        create_dir(store_path)?;
        Ok(())
    }
}

pub fn list(path: &Path) -> Result<()> {
    let dir = path.read_dir()?;

    for entry in dir {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            list(&entry.path())?;
        } else {
            println!("{}", entry.path().display())
        }
    }

    Ok(())
}

pub fn insert(
    store_path: &Path,
    pass_name: &str,
    password: &str,
    passphrase: &str,
    echo: bool,
) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name);

    if !password_path.try_exists()? {
        let encrypted_pass = encrypt(password.as_bytes(), passphrase)?;

        let mut password_file = File::create_new(password_path.as_path())?;

        password_file.write_all(encrypted_pass.as_slice())?;

        if echo {
            println!("{password}");
            Ok(())
        } else {
            Ok(())
        }
    } else {
        Err(anyhow!(
            "A file with the same name already exists at {0}",
            password_path.display()
        ))
    }
}

pub fn show(store_path: &Path, pass_name: &str, passphrase: &str, clipboard: bool) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name);

    let file_contents = read(password_path)?;

    let password = String::from_utf8(decrypt(file_contents.as_slice(), passphrase)?)?;

    if clipboard {
        send_to_clipboard(&password)
    } else {
        println!("{}", password);
        Ok(())
    }
}

pub fn edit(store_path: &Path, pass_name: &str, passphrase: &str) -> Result<()> {
    let editor = if let Ok(editor) = env::var("EDITOR") {
        editor
    } else if let Ok(editor) = env::var("VISUAL") {
        editor
    } else {
        String::from("nano")
    };

    let password_path = get_password_path(store_path, pass_name);

    if password_path.try_exists()? {
        let mut temp_file = NamedTempFile::new()?;

        let password = decrypt(read(password_path.as_path())?.as_slice(), passphrase)?;

        temp_file.write_all(password.as_slice())?;

        let status = Command::new(editor).arg(temp_file.path()).status()?;

        ensure!(status.success());

        let edited_password = read(temp_file.path())?;

        let encrypted_new_password = encrypt(edited_password.trim_ascii(), passphrase)?;

        let mut pass_file = File::create(password_path.as_path())?;

        pass_file.write_all(encrypted_new_password.as_slice())?;

        Ok(())
    } else {
        Err(anyhow!(
            "The password {pass_name} was not found in the store"
        ))
    }
}

pub fn remove(store_path: &Path, pass_name: &str) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name);

    remove_file(&password_path)?;

    Ok(())
}

pub fn find_password(store_path: &Path, pass_names: &[String]) {
    pass_names.iter().for_each(|pass_name| {
        read_dir(store_path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().ok())
            .filter(|file_name| file_name.is_some())
            .map(|file_name| file_name.unwrap())
            .for_each(|name| {
                if name.contains(pass_name) {
                    println!("{name}")
                }
            });
    });
}

fn send_to_clipboard(content: &str) -> Result<()> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        let status = Command::new("wl-copy").arg(content).status()?;

        ensure!(status.success());

        Ok(())
    } else {
        let echo = Command::new("echo")
            .arg(content)
            .stdout(Stdio::piped())
            .spawn()?;

        let status = Command::new("xclip")
            .arg("-i")
            .stdin(Stdio::from(echo.stdout.unwrap()))
            .status()?;

        ensure!(status.success());

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
