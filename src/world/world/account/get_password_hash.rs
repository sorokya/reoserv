use crate::SETTINGS;
use sha2::{Digest, Sha256};

pub fn get_password_hash(username: &str, password: &str) -> String {
    let hash_input = format!("{}{}{}", SETTINGS.server.password_salt, username, password);
    let hash = Sha256::digest(hash_input.as_bytes());
    format!("{:x}", hash)
}
