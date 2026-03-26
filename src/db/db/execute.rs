use super::super::{Db, PreparedQuery};
use crate::db::{db::is_mysql_connection_closed, params::sqlite_named_params};

impl Db {
    pub async fn execute(&mut self, query: &str) -> anyhow::Result<()> {
        let result = self.execute_inner(query).await;
        self.with_transaction_rollback(result).await
    }

    pub async fn execute_prepared(&mut self, query: &PreparedQuery) -> anyhow::Result<()> {
        let result = self.execute_prepared_inner(query).await;
        self.with_transaction_rollback(result).await
    }

    pub(super) async fn execute_inner(&mut self, query: &str) -> anyhow::Result<()> {
        match self.try_execute_inner(query).await {
            Err(e) if is_mysql_connection_closed(&e) => {
                self.reconnect_mysql().await?;
                self.try_execute_inner(query).await
            }
            result => result,
        }
    }

    async fn try_execute_inner(&mut self, query: &str) -> anyhow::Result<()> {
        let last_insert_id_update = match self.connection {
            crate::db::Connection::Mysql(ref mut mysql) => {
                Self::execute_mysql_raw(&mut mysql.conn, query).await?;
                Some(mysql.conn.last_insert_id())
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                Self::execute_sqlite_raw(conn, query)?;
                let last_id = conn.last_insert_rowid();
                if last_id > 0 {
                    Some(Some(last_id as u64))
                } else {
                    None
                }
            }
        };

        if let Some(last_insert_id) = last_insert_id_update {
            self.last_insert_id = last_insert_id;
        }

        Ok(())
    }

    async fn execute_prepared_inner(&mut self, query: &PreparedQuery) -> anyhow::Result<()> {
        match self.try_execute_prepared_inner(query).await {
            Err(e) if is_mysql_connection_closed(&e) => {
                self.reconnect_mysql().await?;
                self.try_execute_prepared_inner(query).await
            }
            result => result,
        }
    }

    async fn try_execute_prepared_inner(&mut self, query: &PreparedQuery) -> anyhow::Result<()> {
        let last_insert_id_update = match self.connection {
            crate::db::Connection::Mysql(ref mut mysql) => {
                Self::execute_mysql_prepared(&mut mysql.conn, query).await?;
                Some(mysql.conn.last_insert_id())
            }
            crate::db::Connection::Sqlite(ref mut conn) => {
                Self::execute_sqlite_prepared(conn, query)?;
                let last_id = conn.last_insert_rowid();
                if last_id > 0 {
                    Some(Some(last_id as u64))
                } else {
                    None
                }
            }
        };

        if let Some(last_insert_id) = last_insert_id_update {
            self.last_insert_id = last_insert_id;
        }

        Ok(())
    }

    async fn execute_mysql_raw(conn: &mut mysql_async::Conn, query: &str) -> anyhow::Result<()> {
        mysql_async::prelude::Queryable::query_drop(conn, query).await?;
        Ok(())
    }

    async fn execute_mysql_prepared(
        conn: &mut mysql_async::Conn,
        query: &PreparedQuery,
    ) -> anyhow::Result<()> {
        mysql_async::prelude::Queryable::exec_drop(
            conn,
            &query.query,
            query.params.to_mysql_params(),
        )
        .await?;
        Ok(())
    }

    fn execute_sqlite_raw(conn: &mut rusqlite::Connection, query: &str) -> anyhow::Result<()> {
        conn.execute_batch(query)?;
        Ok(())
    }

    fn execute_sqlite_prepared(
        conn: &mut rusqlite::Connection,
        query: &PreparedQuery,
    ) -> anyhow::Result<()> {
        let mut stmt = conn.prepare(&query.query)?;
        let sqlite_values = query.params.to_sqlite_values();
        let named_params = sqlite_named_params(&sqlite_values);
        stmt.execute(named_params.as_slice())?;
        Ok(())
    }
}
