pub fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first_letter) = chars.next() else {
        return String::new();
    };

    first_letter.to_uppercase().chain(chars).collect()
}

#[cfg(test)]
mod tests {
    use super::capitalize;

    #[test]
    fn capitalizes_multibyte_first_character() {
        assert_eq!(capitalize("éclair"), "Éclair");
    }
}
