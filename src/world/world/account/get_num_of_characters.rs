use mysql_async::{prelude::*, Conn, Params, Row};

pub async fn get_num_of_characters(
    conn: &mut Conn,
    account_id: i32,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    match conn
        .exec_first::<Row, &str, Params>(
            r"SELECT COUNT(id) FROM `Character` WHERE `account_id` = :account_id",
            params! {
                "account_id" => account_id,
            },
        )
        .await?
    {
        Some(row) => {
            let count: usize = row.get(0).unwrap();
            Ok(count)
        }
        _ => Ok(0),
    }
}
