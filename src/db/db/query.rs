use crate::db::row::SqlValue;

use super::super::{Db, Row};

impl Db {
    pub async fn query(&mut self, query: String) -> anyhow::Result<Vec<Row>> {
        let mut rows = Vec::new();

        match self.connection {
            crate::db::Connection::Mysql(ref mut conn) => {
                let result = mysql_async::prelude::Queryable::query_iter(conn, query).await?;
                result
                    .for_each_and_drop(|row| {
                        let mut r = Row::new();
                        for i in 0..row.len() {
                            match row.as_ref(i) {
                                Some(mysql_async::Value::NULL) => r.columns.push(SqlValue::Null),
                                Some(mysql_async::Value::Bytes(buf)) => {
                                    match String::from_utf8(buf.clone()) {
                                        Ok(s) => r.columns.push(SqlValue::String(s)),
                                        Err(_) => {
                                            error!(
                                                "Failed to parse bytes as UTF-8 string: {:?}",
                                                buf
                                            );
                                            r.columns.push(SqlValue::Null);
                                        }
                                    }
                                }
                                Some(mysql_async::Value::Int(n)) => {
                                    r.columns.push(SqlValue::Int(*n as i32))
                                }
                                Some(mysql_async::Value::UInt(n)) => {
                                    r.columns.push(SqlValue::Int(*n as i32))
                                }
                                Some(mysql_async::Value::Date(
                                    year,
                                    month,
                                    day,
                                    hour,
                                    minutes,
                                    seconds,
                                    micro_seconds,
                                )) => {
                                    let date = chrono::NaiveDate::from_ymd_opt(
                                        *year as i32,
                                        *month as u32,
                                        *day as u32,
                                    )
                                    .and_then(|d| {
                                        d.and_hms_micro_opt(
                                            *hour as u32,
                                            *minutes as u32,
                                            *seconds as u32,
                                            *micro_seconds,
                                        )
                                    });
                                    match date {
                                        Some(d) => r.columns.push(SqlValue::Date(d)),
                                        None => {
                                            error!(
                                                "Failed to parse date: {}-{}-{} {}:{}:{}.{}",
                                                year,
                                                month,
                                                day,
                                                hour,
                                                minutes,
                                                seconds,
                                                micro_seconds
                                            );
                                            r.columns.push(SqlValue::Null);
                                        }
                                    }
                                }
                                _ => {
                                    error!("Unsupported MySQL value type: {:?}", row.as_ref(i));
                                    r.columns.push(SqlValue::Null);
                                }
                            }
                        }
                        rows.push(r);
                    })
                    .await?;
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                let mut stmt = conn.prepare(&query)?;
                let column_count = stmt.column_count();
                let mut result = stmt.query([])?;
                while let Some(row) = result.next()? {
                    let mut r = Row::new();
                    for i in 0..column_count {
                        match row.get_ref(i)? {
                            rusqlite::types::ValueRef::Null => r.columns.push(SqlValue::Null),
                            rusqlite::types::ValueRef::Text(buf) => {
                                match String::from_utf8(buf.to_vec()) {
                                    Ok(s) => {
                                        // check if the string is a valid date
                                        if let Ok(date) = chrono::NaiveDateTime::parse_from_str(
                                            &s,
                                            "%Y-%m-%d %H:%M:%S",
                                        ) {
                                            r.columns.push(SqlValue::Date(date));
                                        } else {
                                            r.columns.push(SqlValue::String(s));
                                        }
                                    }
                                    Err(_) => {
                                        error!("Failed to parse bytes as UTF-8 string: {:?}", buf);
                                        r.columns.push(SqlValue::Null);
                                    }
                                }
                            }
                            rusqlite::types::ValueRef::Integer(n) => {
                                r.columns.push(SqlValue::Int(n as i32))
                            }
                            rusqlite::types::ValueRef::Blob(buf) => {
                                match String::from_utf8(buf.to_vec()) {
                                    Ok(s) => r.columns.push(SqlValue::String(s)),
                                    Err(_) => {
                                        error!("Failed to parse bytes as UTF-8 string: {:?}", buf);
                                        r.columns.push(SqlValue::Null);
                                    }
                                }
                            }
                            _ => {
                                error!("Unsupported SQLite value type: {:?}", row.get_ref(i)?);
                                r.columns.push(SqlValue::Null);
                            }
                        }
                    }
                    rows.push(r);
                }
            }
        }

        Ok(rows)
    }
}
