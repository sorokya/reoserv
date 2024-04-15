pub fn mask_email(email: &str) -> String {
    let mut masked = String::with_capacity(email.len());
    let position_of_at = email.chars().position(|c| c == '@').unwrap();
    for (i, c) in email.chars().enumerate() {
        if i == 0 || i >= position_of_at - 1 {
            masked.push(c);
        } else {
            masked.push('*');
        }
    }

    masked
}
