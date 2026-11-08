use anyhow::{Context, Result, anyhow, ensure};
use std::{
    env::{self},
    fs::{File, create_dir, read, read_dir, remove_file},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

use crate::{decrypt, encrypt};

pub fn get_password_path(store: &Path, pass_name: &str) -> Result<PathBuf> {
    let relative = Path::new(pass_name);

    ensure!(
        !relative.as_os_str().is_empty(),
        "Password name can't be empty"
    );

    let mut safe_name = PathBuf::new();

    for component in relative.components() {
        match component {
            std::path::Component::Normal(part) => safe_name.push(part),
            _ => {
                return Err(anyhow!(
                    "Password name must be a relative path without a '.' or '..'"
                ));
            }
        }
    }

    ensure!(!safe_name.as_os_str().is_empty());

    Ok(store.join(safe_name).with_added_extension("age"))
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

pub fn list(store: &Path) -> Result<()> {
    let root = store.canonicalize()?;

    let mut names = Vec::new();

    collect_passwords(&root, &root, &mut names)?;

    names.sort();

    println!("Password store/");

    names.iter().for_each(|name| println!("{name}"));

    Ok(())
}

fn collect_passwords(root: &Path, dir: &Path, names: &mut Vec<String>) -> Result<()> {
    for entry in read_dir(dir)? {
        let entry = entry?;

        let kind = entry.file_type()?;

        let path = entry.path();

        if kind.is_symlink() {
            return Err(anyhow!(
                "Symlinks are not supported in the password store: {}",
                path.display()
            ));
        } else if kind.is_dir() {
            collect_passwords(root, &path, names)?;
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext.eq("age")) {
            let relative = path.strip_prefix(root)?.with_extension("");

            names.push(
                relative
                    .to_str()
                    .context("Password name is not a valid UTF-8")?
                    .to_owned(),
            );
        }
    }

    Ok(())
}

pub fn insert(store_path: &Path, pass_name: &str, echo: bool) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name)?;

    ensure!(
        !password_path.try_exists()?,
        "A file with the same name already exists at {0}",
        password_path.display()
    );

    let passphrase = get_passphrase()?;

    let password = get_password()?;

    let encrypted_pass = encrypt(password.as_bytes(), &passphrase)?;

    let mut password_file = File::create_new(password_path.as_path())?;

    password_file.write_all(encrypted_pass.as_slice())?;

    if echo {
        println!("{password}");
        Ok(())
    } else {
        Ok(())
    }
}

pub fn show(store_path: &Path, pass_name: &str, clipboard: bool) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name)?;

    ensure!(
        password_path.try_exists()?,
        "Password {pass_name} doesn't exist in the store"
    );

    let file_contents = read(password_path)?;

    let passphrase = get_passphrase()?;

    let password = String::from_utf8(decrypt(file_contents.as_slice(), &passphrase)?)?;

    if clipboard {
        send_to_clipboard(&password)
    } else {
        println!("{}", password);
        Ok(())
    }
}

pub fn edit(store_path: &Path, pass_name: &str) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name)?;

    ensure!(
        password_path.try_exists()?,
        "The password {pass_name} was not found in the store"
    );

    let editor = if let Ok(editor) = env::var("EDITOR") {
        editor
    } else if let Ok(editor) = env::var("VISUAL") {
        editor
    } else {
        String::from("nano")
    };

    let passphrase = get_passphrase()?;

    let mut temp_file = NamedTempFile::new()?;

    let password = decrypt(read(password_path.as_path())?.as_slice(), &passphrase)?;

    temp_file.write_all(password.as_slice())?;

    let status = Command::new(editor).arg(temp_file.path()).status()?;

    ensure!(status.success());

    let edited_password = read(temp_file.path())?;

    let encrypted_new_password = encrypt(edited_password.trim_ascii(), &passphrase)?;

    let mut pass_file = File::create(password_path.as_path())?;

    pass_file.write_all(encrypted_new_password.as_slice())?;

    Ok(())
}

pub fn remove(store_path: &Path, pass_name: &str) -> Result<()> {
    let password_path = get_password_path(store_path, pass_name)?;

    remove_file(&password_path)?;

    Ok(())
}

pub fn find_password(store_path: &Path, pass_names: &[String]) {
    pass_names.iter().for_each(|pass_name| {
        read_dir(store_path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().ok())
            .filter(|file_name| file_name.is_some())
            .flatten()
            .for_each(|name| {
                if name.contains(pass_name) {
                    println!("{name}")
                }
            });
    });
}

fn get_passphrase() -> Result<String> {
    let passphrase = rpassword::prompt_password("Type your passphrase: ")?;

    Ok(passphrase)
}

fn get_password() -> Result<String> {
    let password = rpassword::prompt_password("Type your password: ")?;
    let password_confirmation = rpassword::prompt_password("Retype your password: ")?;

    ensure!(
        password == password_confirmation,
        "The typed passwords don't match each other"
    );

    Ok(password.to_owned())
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
    #[test]
    fn test_clipboard() -> anyhow::Result<()> {
        let content = "Hello";

        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            super::send_to_clipboard(content)?;

            let clipboard_content = std::process::Command::new("wl-paste").output()?;

            assert!(clipboard_content.status.success());

            assert_eq!(clipboard_content.stdout.trim_ascii(), content.as_bytes());

            Ok(())
        } else {
            super::send_to_clipboard(content)?;

            let clipboard_content = std::process::Command::new("xclip").arg("-o").output()?;

            assert!(clipboard_content.status.success());

            assert_eq!(clipboard_content.stdout.trim_ascii(), content.as_bytes());

            Ok(())
        }
    }
}
