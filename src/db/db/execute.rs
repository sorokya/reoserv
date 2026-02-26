use super::super::Db;

impl Db {
    pub async fn execute(&mut self, query: &str) -> anyhow::Result<()> {
        if let Err(e) = self.execute_inner(query).await {
            if self.transaction_active {
                if let Err(e) = self.execute_inner("ROLLBACK").await {
                    error!("Failed to rollback transaction: {}", e);
                }
                self.transaction_active = false;
            }
            Err(e)
        } else {
            Ok(())
        }
    }

    pub(super) async fn execute_inner(&mut self, query: &str) -> anyhow::Result<()> {
        match self.connection {
            crate::db::Connection::Mysql(ref mut conn) => {
                mysql_async::prelude::Queryable::query_drop(conn, query).await?;
                self.last_insert_id = conn.last_insert_id();
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                conn.execute_batch(query)?;
                let last_id = conn.last_insert_rowid();
                if last_id > 0 {
                    self.last_insert_id = Some(last_id as u64);
                }
            }
        }
        Ok(())
    }
}
