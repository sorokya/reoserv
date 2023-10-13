pub fn capitalize(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let first_letter_upper_case = word.chars().next().unwrap().to_uppercase();

    format!("{}{}", first_letter_upper_case, &word[1..])
}
