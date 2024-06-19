use axum::{
    extract::State,
    http::{header::SET_COOKIE, StatusCode},
    response::{AppendHeaders, IntoResponse},
    Json,
};
use mysql_async::{params, prelude::Queryable, Params, Pool, Row};

use crate::{
    api::{generate_access_token::generate_access_token, AppError},
    utils::validate_password,
};

pub async fn login(
    State(pool): State<Pool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.get_conn().await?;

    let row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../../sql/get_password_hash.sql"),
            params! {
                "name" => &payload.username,
            },
        )
        .await?
    {
        Some(row) => row,
        None => {
            // Check a hash anyway
            validate_password(&payload.username, &payload.password, "$argon2id$v=19$m=19456,t=2,p=1$2fxYwlgtiSkaQwpuTsFLUg$G43qDEoUMmXRtZX2GBSAD9pVI5wDtSxohb0LgsqgWR0");
            return Ok((StatusCode::FORBIDDEN, "Unauthorized").into_response());
        }
    };

    let account_id: i32 = row.get("id").unwrap();
    let password_hash: String = row.get("password_hash").unwrap();
    if !validate_password(&payload.username, &payload.password, &password_hash) {
        return Ok((StatusCode::FORBIDDEN, "Unauthorized").into_response());
    }

    let access_token = generate_access_token();

    conn.exec_drop(
        include_str!("../../sql/create_access_token.sql"),
        params! {
            "account_id" => &account_id,
            "token" => &access_token,
        },
    )
    .await?;

    Ok((
        StatusCode::OK,
        AppendHeaders([(
            SET_COOKIE,
            format!(
                "access_token={}; Max-Age=1200; Secure; HttpOnly",
                access_token
            ),
        )]),
        String::from("authenticated"),
    )
        .into_response())
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}
