use super::super::Db;

impl Db {
    pub async fn start_transaction(&mut self) -> anyhow::Result<()> {
        let query = match self.connection {
            crate::db::Connection::Mysql(_) => "START TRANSACTION",
            crate::db::Connection::Sqlite(_) => "BEGIN TRANSACTION",
        };
        self.execute(query).await?;
        self.transaction_active = true;
        Ok(())
    }

    pub async fn commit_transaction(&mut self) -> anyhow::Result<()> {
        if !self.transaction_active {
            return Err(anyhow::anyhow!("No active transaction to commit"));
        }
        self.execute("COMMIT").await?;
        self.transaction_active = false;
        Ok(())
    }

    pub async fn rollback_transaction(&mut self) -> anyhow::Result<()> {
        if !self.transaction_active {
            return Err(anyhow::anyhow!("No active transaction to rollback"));
        }
        self.execute("ROLLBACK").await?;
        self.transaction_active = false;
        Ok(())
    }
}
