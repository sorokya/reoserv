use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::CLASS_DB;

pub async fn get_class_list() -> impl IntoResponse {
    let classes = CLASS_DB
        .classes
        .iter()
        .take_while(|class| class.name != "eof")
        .enumerate()
        .map(|(index, class)| ClassListClass {
            id: index as i32 + 1,
            name: class.name.clone(),
        })
        .collect::<Vec<_>>();
    Json(classes).into_response()
}

pub async fn get_class(Path(id): Path<i32>) -> impl IntoResponse {
    match CLASS_DB.classes.get(id as usize - 1) {
        Some(class) => Json(class).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}

#[derive(Serialize)]
struct ClassListClass {
    id: i32,
    name: String,
}
