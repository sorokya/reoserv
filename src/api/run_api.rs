use anyhow::Result;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{
        header::{self, SET_COOKIE},
        request::Parts,
        StatusCode,
    },
    response::{AppendHeaders, IntoResponse, Response},
    routing::{get, post},
    Json, RequestPartsExt, Router,
};
use axum_extra::{typed_header::TypedHeaderRejectionReason, TypedHeader};
use eolib::protocol::AdminLevel;
use mysql_async::{params, prelude::Queryable, Params, Pool, Row};
use tokio::net::TcpListener;

use crate::utils::validate_password;

use super::generate_access_token::generate_access_token;

pub async fn run_api(pool: Pool) {
    let app_state = AppState { pool };

    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login))
        .route("/user", get(user))
        .with_state(app_state);

    let listener = match TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind api listener: {}", e);
            return;
        }
    };

    info!("API Listening at http://localhost:3000");

    if let Err(e) = axum::serve(listener, app).await {
        error!("Failed to start axum serve: {}", e);
    }
}

#[derive(Clone)]
struct AppState {
    pool: Pool,
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn user(user: User) -> impl IntoResponse {
    Json(user)
}

async fn login(
    State(pool): State<Pool>,
    Json(payload): Json<Login>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.get_conn().await?;

    let row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../sql/get_password_hash.sql"),
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
        include_str!("../sql/create_access_token.sql"),
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
struct Login {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    admin_level: AdminLevel,
}

struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    Pool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => AuthError,
                    _ => panic!("unexpected error getting Cookie header(s): {e}"),
                },
                _ => panic!("unexpected error getting cookies: {e}"),
            })?;

        let access_token = cookies.get("access_token").ok_or(AuthError)?;

        let mut conn = match pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return Err(AuthError);
            }
        };

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../sql/get_user_from_access_token.sql"),
                params! {
                    "access_token" => &access_token,
                },
            )
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                return Err(AuthError);
            }
            Err(e) => {
                error!("Error getting user: {}", e);
                return Err(AuthError);
            }
        };

        let user = User {
            id: row.get("id").unwrap(),
            username: row.get("name").unwrap(),
            admin_level: AdminLevel::from(row.get::<i32, &str>("admin_level").unwrap()),
        };

        Ok(user)
    }
}

#[derive(Debug)]
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Application error: {:#}", self.0);

        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
