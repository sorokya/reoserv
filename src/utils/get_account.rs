use mysql_async::{prelude::*, Conn, Params, Row};

pub async fn get_account(
    conn: &mut Conn,
    name: &str,
) -> Result<Option<(u32, String)>, Box<dyn std::error::Error>> {
    let account_row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../sql/get_account.sql"),
            params! {
                "name" => name,
            },
        )
        .await?
    {
        Some(row) => row,
        _ => {
            return Ok(None);
        }
    };

    Ok(Some((
        account_row.get(0).unwrap(),
        account_row.get(1).unwrap(),
    )))
}
