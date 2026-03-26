#[derive(Debug)]
pub struct MysqlConnection {
    pub conn: mysql_async::Conn,
    pub url: String,
}

#[derive(Debug)]
pub enum Connection {
    Mysql(MysqlConnection),
    Sqlite(rusqlite::Connection),
}
