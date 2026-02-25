use std::any;

use super::super::Db;

impl Db {
    pub async fn execute(&mut self, query: String) -> anyhow::Result<()> {
        println!("Executing query: {}", query);
        // Here you would execute the query against your database
        Ok(())
    }
}
