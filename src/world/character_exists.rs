use mysql_async::{prelude::*, Conn, Params, Row};

pub async fn character_exists(
    conn: &mut Conn,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    match conn
        .exec_first::<Row, &str, Params>(
            r"SELECT id FROM `Character` WHERE `name` = :name",
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
