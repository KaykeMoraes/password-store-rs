use age::{
    scrypt::{Identity, Recipient},
    secrecy::SecretString,
};
use anyhow::Result;

pub fn encrypt(content: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let secret = SecretString::from(passphrase.to_owned());

    let recipient = Recipient::new(secret.clone());

    let encrypted = age::encrypt(&recipient, content)?;

    Ok(encrypted)
}

pub fn decrypt(encrypted_content: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let secret = SecretString::from(passphrase.to_owned());

    let identity = Identity::new(secret);

    let decrypted_content = age::decrypt(&identity, encrypted_content)?;

    Ok(decrypted_content)
}
