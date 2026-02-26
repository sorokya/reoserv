use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SqlParam {
    Null,
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
    Bytes(Vec<u8>),
}

impl SqlParam {
    pub fn to_mysql_value(&self) -> mysql_async::Value {
        match self {
            Self::Null => mysql_async::Value::NULL,
            Self::String(value) => mysql_async::Value::Bytes(value.clone().into_bytes()),
            Self::Int(value) => mysql_async::Value::Int(*value),
            Self::UInt(value) => mysql_async::Value::UInt(*value),
            Self::Float(value) => mysql_async::Value::Double(*value),
            Self::Bool(value) => mysql_async::Value::Int(i64::from(*value)),
            Self::DateTime(value) => mysql_async::Value::Date(
                value.year() as u16,
                value.month() as u8,
                value.day() as u8,
                value.hour() as u8,
                value.minute() as u8,
                value.second() as u8,
                value.and_utc().timestamp_subsec_micros(),
            ),
            Self::Bytes(value) => mysql_async::Value::Bytes(value.clone()),
        }
    }

    pub fn to_sqlite_value(&self) -> rusqlite::types::Value {
        match self {
            Self::Null => rusqlite::types::Value::Null,
            Self::String(value) => rusqlite::types::Value::Text(value.clone()),
            Self::Int(value) => rusqlite::types::Value::Integer(*value),
            Self::UInt(value) => rusqlite::types::Value::Integer(*value as i64),
            Self::Float(value) => rusqlite::types::Value::Real(*value),
            Self::Bool(value) => rusqlite::types::Value::Integer(i64::from(*value)),
            Self::DateTime(value) => {
                rusqlite::types::Value::Text(value.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            Self::Bytes(value) => rusqlite::types::Value::Blob(value.clone()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SqlParams {
    named: Vec<(String, SqlParam)>,
}

impl SqlParams {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn bind(mut self, key: impl Into<String>, value: impl ToSqlParam) -> Self {
        self.named.push((key.into(), value.to_sql_param()));
        self
    }

    #[allow(dead_code)]
    pub fn push(&mut self, key: impl Into<String>, value: impl ToSqlParam) {
        self.named.push((key.into(), value.to_sql_param()));
    }

    pub fn push_param(&mut self, key: impl Into<String>, value: SqlParam) {
        self.named.push((key.into(), value));
    }

    pub fn to_mysql_params(&self) -> mysql_async::Params {
        let mut values = HashMap::with_capacity(self.named.len());
        for (key, value) in &self.named {
            values.insert(key.clone().into_bytes(), value.to_mysql_value());
        }
        mysql_async::Params::Named(values)
    }

    pub fn to_sqlite_values(&self) -> Vec<(String, rusqlite::types::Value)> {
        self.named
            .iter()
            .map(|(key, value)| (format!(":{key}"), value.to_sqlite_value()))
            .collect()
    }
}

pub fn sqlite_named_params(
    values: &[(String, rusqlite::types::Value)],
) -> Vec<(&str, &dyn rusqlite::ToSql)> {
    values
        .iter()
        .map(|(key, value)| (key.as_str(), value as &dyn rusqlite::ToSql))
        .collect()
}

#[derive(Debug, Clone)]
pub struct PreparedQuery {
    pub query: String,
    pub params: SqlParams,
}

impl PreparedQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            params: SqlParams::new(),
        }
    }

    #[allow(dead_code)]
    pub fn bind(mut self, key: impl Into<String>, value: impl ToSqlParam) -> Self {
        self.params.push(key, value);
        self
    }
}

#[derive(Debug, Clone)]
pub enum DbRequest {
    Raw(String),
    Prepared(PreparedQuery),
}

pub trait IntoDbRequest {
    fn to_db_request(&self) -> DbRequest;
}

impl IntoDbRequest for str {
    fn to_db_request(&self) -> DbRequest {
        DbRequest::Raw(self.to_string())
    }
}

impl IntoDbRequest for String {
    fn to_db_request(&self) -> DbRequest {
        DbRequest::Raw(self.clone())
    }
}

impl IntoDbRequest for PreparedQuery {
    fn to_db_request(&self) -> DbRequest {
        DbRequest::Prepared(self.clone())
    }
}

pub fn insert_params(
    query: &str,
    params: &[(&str, &(dyn ToSqlParam + Send + Sync))],
) -> PreparedQuery {
    let mut query_with_params = PreparedQuery::new(query);
    for (key, value) in params {
        query_with_params
            .params
            .push_param(*key, value.to_sql_param());
    }
    query_with_params
}

pub trait ToSqlParam {
    fn to_sql_param(&self) -> SqlParam;
}

impl ToSqlParam for String {
    fn to_sql_param(&self) -> SqlParam {
        SqlParam::String(self.clone())
    }
}

impl ToSqlParam for &str {
    fn to_sql_param(&self) -> SqlParam {
        SqlParam::String((*self).to_string())
    }
}

impl ToSqlParam for chrono::NaiveDateTime {
    fn to_sql_param(&self) -> SqlParam {
        SqlParam::DateTime(*self)
    }
}

impl ToSqlParam for bool {
    fn to_sql_param(&self) -> SqlParam {
        SqlParam::Bool(*self)
    }
}

impl ToSqlParam for Vec<u8> {
    fn to_sql_param(&self) -> SqlParam {
        SqlParam::Bytes(self.clone())
    }
}

impl ToSqlParam for &[u8] {
    fn to_sql_param(&self) -> SqlParam {
        SqlParam::Bytes(self.to_vec())
    }
}

macro_rules! impl_signed_to_sql_param {
	($($type:ty),* $(,)?) => {
		$(
			impl ToSqlParam for $type {
				fn to_sql_param(&self) -> SqlParam {
					SqlParam::Int(*self as i64)
				}
			}
		)*
	};
}

macro_rules! impl_unsigned_to_sql_param {
	($($type:ty),* $(,)?) => {
		$(
			impl ToSqlParam for $type {
				fn to_sql_param(&self) -> SqlParam {
					SqlParam::UInt(*self as u64)
				}
			}
		)*
	};
}

macro_rules! impl_float_to_sql_param {
	($($type:ty),* $(,)?) => {
		$(
			impl ToSqlParam for $type {
				fn to_sql_param(&self) -> SqlParam {
					SqlParam::Float(*self as f64)
				}
			}
		)*
	};
}

impl_signed_to_sql_param!(i8, i16, i32, i64, isize);
impl_unsigned_to_sql_param!(u8, u16, u32, u64, usize);
impl_float_to_sql_param!(f32, f64);

impl<T> ToSqlParam for Option<T>
where
    T: ToSqlParam,
{
    fn to_sql_param(&self) -> SqlParam {
        match self {
            Some(value) => value.to_sql_param(),
            None => SqlParam::Null,
        }
    }
}

use chrono::{Datelike, Timelike};
