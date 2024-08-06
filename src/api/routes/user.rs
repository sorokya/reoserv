use axum::{response::IntoResponse, Json};

use crate::api::User;

pub async fn user(user: User) -> impl IntoResponse {
    Json(user)
}
