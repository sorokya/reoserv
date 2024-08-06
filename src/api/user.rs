use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use axum_extra::{typed_header::TypedHeaderRejectionReason, TypedHeader};
use eolib::protocol::AdminLevel;
use mysql_async::{params, prelude::Queryable, Pool, Row};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    username: String,
    admin_level: AdminLevel,
    characters: Vec<UserCharacter>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserCharacter {
    id: i32,
    name: String,
}

pub struct AuthError;

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

        let mut user = User::default();

        let characters = match conn
            .exec_map(
                include_str!("../sql/get_user_from_access_token.sql"),
                params! {
                    "access_token" => &access_token,
                },
                |row: Row| {
                    if user.id == 0 {
                        user.id = row.get::<i32, &str>("account_id").unwrap();
                        user.username = row.get::<String, &str>("account_name").unwrap();
                    }
                    let admin_level = row.get::<i32, &str>("admin_level").unwrap();
                    if admin_level > user.admin_level.into() {
                        user.admin_level = AdminLevel::from(admin_level);
                    }

                    UserCharacter {
                        id: row.get::<i32, &str>("character_id").unwrap(),
                        name: row.get::<String, &str>("character_name").unwrap(),
                    }
                },
            )
            .await
        {
            Ok(characters) => characters,
            Err(e) => {
                error!("Error getting user: {}", e);
                return Err(AuthError);
            }
        };

        if user.id == 0 {
            return Err(AuthError);
        }

        user.characters = characters;

        Ok(user)
    }
}
