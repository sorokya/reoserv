use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::world::WorldHandle;

pub async fn get_map_list(State(world): State<WorldHandle>) -> impl IntoResponse {
    let maps = world.get_map_list().await;
    Json(maps).into_response()
}

pub async fn get_map(Path(id): Path<i32>, State(world): State<WorldHandle>) -> impl IntoResponse {
    match world.get_map(id).await {
        Ok(map) => Json(map.get_state().await).into_response(),
        Err(_) => (StatusCode::NOT_FOUND).into_response(),
    }
}
