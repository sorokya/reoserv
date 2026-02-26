#[derive(Debug)]
pub struct Row {
    pub columns: Vec<SqlValue>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> SqlValue {
        self.columns.get(index).cloned().unwrap_or(SqlValue::Null)
    }

    pub fn get_string(&self, index: usize) -> Option<String> {
        match self.get(index) {
            SqlValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn get_int(&self, index: usize) -> Option<i32> {
        match self.get(index) {
            SqlValue::Int(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_date(&self, index: usize) -> Option<chrono::NaiveDateTime> {
        match self.get(index) {
            SqlValue::Date(d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlValue {
    Null,
    String(String),
    Int(i32),
    Date(chrono::NaiveDateTime),
}
