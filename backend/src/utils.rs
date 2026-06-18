use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{self, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Deserializer};
use tokio::task;

pub fn trimmed_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let s = String::deserialize(d)?;
    Ok(s.trim().to_string())
}

pub fn trimmed_option<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let opt = Option::<String>::deserialize(d)?;

    let cleaned = opt.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

    Ok(cleaned)
}

pub async fn password_hash(password: String) -> Result<String, password_hash::Error> {
    task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .unwrap_or(Err(password_hash::Error::Crypto))
}

pub async fn password_verify(
    password: String,
    hash_str: String,
) -> Result<bool, argon2::password_hash::Error> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash_str)?;

        let argon2 = Argon2::default();
        let verification_result = argon2.verify_password(password.as_bytes(), &parsed_hash);

        match verification_result {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    })
    .await
    .unwrap_or(Err(argon2::password_hash::Error::Crypto))
}
