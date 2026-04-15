static EMAIL_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    // Simple regex to validate email format
    regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("email validation regex must be valid")
});

pub fn normalize_email(email: &str) -> anyhow::Result<String> {
    // Simple regex to validate email format
    if !EMAIL_REGEX.is_match(email) {
        anyhow::bail!("Invalid email format");
    }
    Ok(email.to_lowercase())
}
