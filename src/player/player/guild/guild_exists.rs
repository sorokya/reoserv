use mysql_async::{prelude::Queryable, Conn, Params};
use mysql_common::{params, Row};

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
