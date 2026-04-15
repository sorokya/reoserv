pub fn normalize_email(email: &str) -> anyhow::Result<String> {
    // Simple regex to validate email format
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")?;
    if !email_regex.is_match(email) {
        anyhow::bail!("Invalid email format");
    }
    Ok(email.to_lowercase())
}
