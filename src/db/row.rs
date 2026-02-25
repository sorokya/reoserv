#[derive(Debug)]
pub struct Row {
    pub columns: Vec<String>,
}

impl Row {
    pub fn get(&self, index: usize) -> Option<&str> {
        self.columns.get(index).map(|s| s.as_str())
    }

    pub fn get_as<T: std::str::FromStr>(&self, index: usize) -> Option<T> {
        self.get(index)?.parse().ok()
    }
}
