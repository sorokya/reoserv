use mysql_async::{prelude::Queryable, Conn};
use mysql_common::{params, Row};

pub async fn get_guild_ranks(conn: &mut Conn, tag: &str) -> Vec<String> {
    match conn
        .exec_map(
            include_str!("../sql/get_guild_ranks.sql"),
            params! {
                "tag" => tag,
            },
            |row: Row| row.get::<String, usize>(0).unwrap(),
        )
        .await
    {
        Ok(ranks) => ranks,
        Err(e) => {
            error!("Error getting guild ranks: {}", e);
            vec!["".to_owned(); 9]
        }
    }
}
