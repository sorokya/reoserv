#[derive(Debug)]
pub enum Connection {
    Mysql(mysql_async::Conn),
    Sqlite(rusqlite::Connection),
}
