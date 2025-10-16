use std::fmt::Display;
use std::str::FromStr;

use serde::de::Error;
use serde::{Deserialize, Deserializer};
use argon2::{password_hash::SaltString, Argon2, PasswordHash};

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber<T> {
    String(String),
    Number(T),
}

pub fn deserialize_number<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + Deserialize<'de>,
    T::Err: Display,
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse().map_err(Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(rand::thread_rng());
    Ok(PasswordHash::generate(Argon2::default(), password, &salt)
        .map_err(|e| anyhow::anyhow!("failed to generate password hash: {}", e))?
        .to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> anyhow::Result<()> {
    let hash = PasswordHash::new(&password_hash)
        .map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;
    let result = hash.verify_password(&[&Argon2::default()], password);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!("invalid password")),
    }
}
