use super::super::Db;

impl Db {
    pub async fn execute(&mut self, query: String) -> anyhow::Result<()> {
        match self.connection {
            crate::db::Connection::Mysql(ref mut conn) => {
                mysql_async::prelude::Queryable::query_drop(conn, query).await?;
                self.last_insert_id = conn.last_insert_id();
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                conn.execute_batch(&query)?;
                let last_id = conn.last_insert_rowid();
                if last_id > 0 {
                    self.last_insert_id = Some(last_id as u64);
                }
            }
        }
        Ok(())
    }
}
