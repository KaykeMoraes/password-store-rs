use anyhow::Result;

#[test]
fn encryption_works() -> Result<()> {
    let passphrase = "passphrase_example";

    let content = b"Password_example";

    pass::encrypt(content, passphrase)?;

    Ok(())
}

#[test]
fn decryption_works() -> Result<()> {
    let passphrase = "passphrase_example";

    let content = b"Password_example";

    let encrypted_content = pass::encrypt(content, passphrase)?;

    let decrypted_content = pass::decrypt(&encrypted_content, passphrase)?;

    assert_eq!(content, &decrypted_content[..]);

    Ok(())
}
