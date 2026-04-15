pub fn truncate_to_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::truncate_to_chars;

    #[test]
    fn truncates_on_char_boundary() {
        assert_eq!(
            truncate_to_chars("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAé", 31),
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
    }

    #[test]
    fn keeps_multibyte_char_when_it_fits() {
        assert_eq!(truncate_to_chars("café", 4), "café");
    }
}
