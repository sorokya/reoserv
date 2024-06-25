use axum::{
    http::{header::SET_COOKIE, StatusCode},
    response::{AppendHeaders, IntoResponse},
};

pub async fn logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        AppendHeaders([(
            SET_COOKIE,
            format!("access_token=; Max-Age=1; Secure; HttpOnly; SameSite=Lax"),
        )]),
        "logged out",
    )
}
