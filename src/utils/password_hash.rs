use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

pub fn generate_password_hash(username: &str, password: &str) -> String {
    let hash_input = format!("{}{}", username, password);

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(hash_input.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn validate_password(username: &str, password: &str, password_hash: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    let hash_input = format!("{}{}", username, password);
    argon2
        .verify_password(hash_input.as_bytes(), &parsed_hash)
        .is_ok()
}
