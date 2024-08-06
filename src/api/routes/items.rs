use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::ITEM_DB;

pub async fn get_item_list() -> impl IntoResponse {
    let items = ITEM_DB
        .items
        .iter()
        .take_while(|item| item.name != "eof")
        .enumerate()
        .map(|(index, item)| ItemListItem {
            id: index as i32 + 1,
            name: item.name.clone(),
        })
        .collect::<Vec<_>>();
    Json(items).into_response()
}

pub async fn get_item(Path(id): Path<i32>) -> impl IntoResponse {
    match ITEM_DB.items.get(id as usize - 1) {
        Some(item) => Json(item).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}

#[derive(Serialize)]
struct ItemListItem {
    id: i32,
    name: String,
}
