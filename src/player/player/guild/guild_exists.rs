use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

pub async fn guild_exists(conn: &mut Conn, guild_tag: &str, guild_name: &str) -> bool {
    matches!(
        conn.exec_first::<Row, &str, Params>(
            "SELECT id FROM Guild WHERE name = :name OR tag = :tag",
            params! {
                "name" => guild_name,
                "tag" => guild_tag,
            },
        )
        .await,
        Ok(Some(_))
    )
}
