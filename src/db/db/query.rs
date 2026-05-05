use crate::db::row::SqlValue;

use super::super::{Db, PreparedQuery, Row};
use crate::db::{db::is_mysql_connection_closed, params::sqlite_named_params};

impl Db {
    pub async fn query(&mut self, query: &str) -> anyhow::Result<Vec<Row>> {
        let result = self.query_inner(query).await;
        self.with_transaction_rollback(result).await
    }

    pub async fn query_prepared(&mut self, query: &PreparedQuery) -> anyhow::Result<Vec<Row>> {
        let result = self.query_prepared_inner(query).await;
        self.with_transaction_rollback(result).await
    }

    async fn query_inner(&mut self, query: &str) -> anyhow::Result<Vec<Row>> {
        match self.try_query_inner(query).await {
            Err(e) if is_mysql_connection_closed(&e) => {
                self.reconnect_mysql().await?;
                self.try_query_inner(query).await
            }
            result => result,
        }
    }

    async fn try_query_inner(&mut self, query: &str) -> anyhow::Result<Vec<Row>> {
        let mut rows = Vec::new();

        match self.connection {
            crate::db::Connection::Mysql(ref mut mysql) => {
                let result =
                    mysql_async::prelude::Queryable::query_iter(&mut mysql.conn, query).await?;
                result
                    .for_each_and_drop(|row| rows.push(map_mysql_row(row)))
                    .await?;
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                let mut stmt = conn.prepare(query)?;
                let column_count = stmt.column_count();
                let mut result = stmt.query([])?;
                rows.extend(read_sqlite_rows(&mut result, column_count)?);
            }
        }

        Ok(rows)
    }

    async fn query_prepared_inner(&mut self, query: &PreparedQuery) -> anyhow::Result<Vec<Row>> {
        match self.try_query_prepared_inner(query).await {
            Err(e) if is_mysql_connection_closed(&e) => {
                self.reconnect_mysql().await?;
                self.try_query_prepared_inner(query).await
            }
            result => result,
        }
    }

    async fn try_query_prepared_inner(
        &mut self,
        query: &PreparedQuery,
    ) -> anyhow::Result<Vec<Row>> {
        let mut rows = Vec::new();

        match self.connection {
            crate::db::Connection::Mysql(ref mut mysql) => {
                let result = mysql_async::prelude::Queryable::exec_iter(
                    &mut mysql.conn,
                    &query.query,
                    query.params.to_mysql_params(),
                )
                .await?;

                result
                    .for_each_and_drop(|row| rows.push(map_mysql_row(row)))
                    .await?;
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                let mut stmt = conn.prepare(&query.query)?;
                let sqlite_values = query.params.to_sqlite_values();
                let named_params = sqlite_named_params(&sqlite_values);
                let column_count = stmt.column_count();
                let mut result = stmt.query(named_params.as_slice())?;
                rows.extend(read_sqlite_rows(&mut result, column_count)?);
            }
        }

        Ok(rows)
    }
}

fn map_mysql_row(row: mysql_async::Row) -> Row {
    let mut mapped_row = Row::new();
    for column_index in 0..row.len() {
        mapped_row
            .columns
            .push(map_mysql_value(row.as_ref(column_index)));
    }
    mapped_row
}

fn map_mysql_value(value: Option<&mysql_async::Value>) -> SqlValue {
    match value {
        Some(mysql_async::Value::NULL) => SqlValue::Null,
        Some(mysql_async::Value::Bytes(buffer)) => map_utf8_buffer(buffer),
        Some(mysql_async::Value::Int(value)) => SqlValue::Int(*value as i32),
        Some(mysql_async::Value::UInt(value)) => SqlValue::Int(*value as i32),
        Some(mysql_async::Value::Date(year, month, day, hour, minutes, seconds, micro_seconds)) => {
            let parsed = chrono::NaiveDate::from_ymd_opt(*year as i32, *month as u32, *day as u32)
                .and_then(|date| {
                    date.and_hms_micro_opt(
                        *hour as u32,
                        *minutes as u32,
                        *seconds as u32,
                        *micro_seconds,
                    )
                });

            if let Some(date) = parsed {
                SqlValue::Date(date)
            } else {
                tracing::error!(
                    "Failed to parse date: {}-{}-{} {}:{}:{}.{}",
                    year, month, day, hour, minutes, seconds, micro_seconds
                );
                SqlValue::Null
            }
        }
        _ => {
            tracing::error!("Unsupported MySQL value type: {:?}", value);
            SqlValue::Null
        }
    }
}

fn read_sqlite_rows(
    result: &mut rusqlite::Rows<'_>,
    column_count: usize,
) -> anyhow::Result<Vec<Row>> {
    let mut rows = Vec::new();

    while let Some(row) = result.next()? {
        let mut mapped_row = Row::new();
        for column_index in 0..column_count {
            mapped_row
                .columns
                .push(map_sqlite_value(row.get_ref(column_index)?));
        }
        rows.push(mapped_row);
    }

    Ok(rows)
}

fn map_sqlite_value(value: rusqlite::types::ValueRef<'_>) -> SqlValue {
    match value {
        rusqlite::types::ValueRef::Null => SqlValue::Null,
        rusqlite::types::ValueRef::Text(buffer) => map_utf8_buffer(buffer),
        rusqlite::types::ValueRef::Integer(value) => SqlValue::Int(value as i32),
        rusqlite::types::ValueRef::Blob(buffer) => map_blob_as_string(buffer),
        unsupported => {
            tracing::error!("Unsupported SQLite value type: {:?}", unsupported);
            SqlValue::Null
        }
    }
}

fn map_utf8_buffer(buffer: &[u8]) -> SqlValue {
    match String::from_utf8(buffer.to_vec()) {
        Ok(value) => {
            if let Ok(int) = value.parse::<i32>() {
                SqlValue::Int(int)
            } else if let Ok(date) =
                chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")
            {
                SqlValue::Date(date)
            } else {
                SqlValue::String(value)
            }
        }
        Err(_) => {
            tracing::error!("Failed to parse bytes as UTF-8 string: {:?}", buffer);
            SqlValue::Null
        }
    }
}

fn map_blob_as_string(buffer: &[u8]) -> SqlValue {
    match String::from_utf8(buffer.to_vec()) {
        Ok(value) => SqlValue::String(value),
        Err(_) => {
            tracing::error!("Failed to parse bytes as UTF-8 string: {:?}", buffer);
            SqlValue::Null
        }
    }
}
