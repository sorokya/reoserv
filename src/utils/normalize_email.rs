use emval::{ValidationError, validate_email};

pub fn normalize_email(email: &str) -> Result<String, ValidationError> {
    let val_email = validate_email(email)?;
    Ok(val_email.normalized)
}
