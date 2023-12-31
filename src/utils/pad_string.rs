pub fn pad_string(string: &str, length: usize) -> String {
    let mut string = string.to_string();
    while string.len() < length {
        string.push(' ');
    }
    string
}