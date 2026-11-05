use std::error::Error;

#[test]
fn encryption_works() -> Result<(), Box<dyn Error>> {
    let passphrase = "passphrase_example";

    let content = b"Password_example";

    pass::encrypt(content, passphrase)?;

    Ok(())
}

#[test]
fn decryption_works() -> Result<(), Box<dyn Error>> {
    let passphrase = "passphrase_example";

    let content = b"Password_example";

    let encrypted_content = pass::encrypt(content, passphrase)?;

    let decrypted_content = pass::decrypt(&encrypted_content, passphrase)?;

    assert_eq!(content, &decrypted_content[..]);

    Ok(())
}
