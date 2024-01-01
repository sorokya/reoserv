use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

pub async fn account_exists(
    conn: &mut Conn,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match conn
        .exec_first::<Row, &str, Params>(
            r"SELECT id FROM `Account` WHERE `name` = :name",
            params! {
                "name" => name,
            },
        )
        .await?
    {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}
