use argon2::{
    Argon2, PasswordHasher,
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
    Ok(opt.map(|s| s.trim().to_string()))
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
