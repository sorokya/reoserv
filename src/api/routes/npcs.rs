use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::NPC_DB;

pub async fn get_npc_list() -> impl IntoResponse {
    let npcs = NPC_DB
        .npcs
        .iter()
        .take_while(|npc| npc.name != "eof")
        .enumerate()
        .map(|(index, npc)| NpcListNpc {
            id: index as i32 + 1,
            name: npc.name.clone(),
        })
        .collect::<Vec<_>>();
    Json(npcs).into_response()
}

pub async fn get_npc(Path(id): Path<i32>) -> impl IntoResponse {
    match NPC_DB.npcs.get(id as usize - 1) {
        Some(npc) => Json(npc).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}

#[derive(Serialize)]
struct NpcListNpc {
    id: i32,
    name: String,
}
