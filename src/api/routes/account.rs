use axum::{extract::State, response::IntoResponse, Json};
use mysql_async::{params, prelude::Queryable, Params, Pool, Row};

use crate::api::{
    account::Account,
    user::{AuthError, User},
};

pub async fn get_account(
    user: User,
    State(pool): State<Pool>,
) -> Result<impl IntoResponse, AuthError> {
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(AuthError);
        }
    };

    let row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../../sql/get_account.sql"),
            params! {
                "id" => &user.id
            },
        )
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err(AuthError);
        }
        Err(e) => {
            error!("Error getting account: {}", e);
            return Err(AuthError);
        }
    };

    let account = Account {
        id: row.get::<i32, &str>("id").unwrap(),
        username: row.get::<String, &str>("name").unwrap(),
        email: row.get::<String, &str>("email").unwrap(),
        real_name: row.get::<String, &str>("real_name").unwrap(),
        location: row.get::<String, &str>("location").unwrap(),
    };

    Ok(Json(account))
}
