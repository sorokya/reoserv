pub fn insert_params(query: &str, params: &[(&str, &(dyn ToSqlValue + Send + Sync))]) -> String {
    let mut prepared_query = query.to_string();
    for (key, value) in params {
        let placeholder = format!(":{}", key);
        prepared_query = prepared_query.replace(&placeholder, &value.to_sql_value());
    }
    prepared_query
}

pub trait ToSqlValue {
    fn to_sql_value(&self) -> String;
}

impl ToSqlValue for String {
    fn to_sql_value(&self) -> String {
        format!("'{}'", self.replace("'", "''"))
    }
}

impl ToSqlValue for &str {
    fn to_sql_value(&self) -> String {
        format!("'{}'", self.to_string().replace("'", "''"))
    }
}

impl ToSqlValue for i32 {
    fn to_sql_value(&self) -> String {
        self.to_string()
    }
}

impl ToSqlValue for u32 {
    fn to_sql_value(&self) -> String {
        self.to_string()
    }
}

impl ToSqlValue for chrono::NaiveDateTime {
    fn to_sql_value(&self) -> String {
        format!("'{}'", self.format("%Y-%m-%d %H:%M:%S"))
    }
}

impl ToSqlValue for Option<String> {
    fn to_sql_value(&self) -> String {
        match self {
            Some(s) => s.to_sql_value(),
            None => "NULL".to_string(),
        }
    }
}

impl ToSqlValue for Option<&str> {
    fn to_sql_value(&self) -> String {
        match self {
            Some(s) => s.to_sql_value(),
            None => "NULL".to_string(),
        }
    }
}

impl ToSqlValue for Option<i32> {
    fn to_sql_value(&self) -> String {
        match self {
            Some(n) => n.to_sql_value(),
            None => "NULL".to_string(),
        }
    }
}

impl ToSqlValue for Option<u32> {
    fn to_sql_value(&self) -> String {
        match self {
            Some(n) => n.to_sql_value(),
            None => "NULL".to_string(),
        }
    }
}

impl ToSqlValue for Option<chrono::NaiveDateTime> {
    fn to_sql_value(&self) -> String {
        match self {
            Some(d) => d.to_sql_value(),
            None => "NULL".to_string(),
        }
    }
}
