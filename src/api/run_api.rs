use axum::{
    routing::{get, post},
    Router,
};
use mysql_async::Pool;
use tokio::net::TcpListener;

use crate::{
    api::{
        routes::{
            get_item, get_item_list, get_map, get_map_list, get_npc, get_npc_list, login, logout,
            root, user,
        },
        AppState,
    },
    world::WorldHandle,
    SETTINGS,
};

pub async fn run_api(pool: Pool, world: WorldHandle) {
    let app_state = AppState { pool, world };

    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/user", get(user))
        .route("/items/list", get(get_item_list))
        .route("/items/:id", get(get_item))
        .route("/npcs/list", get(get_npc_list))
        .route("/npcs/:id", get(get_npc))
        .route("/maps/list", get(get_map_list))
        .route("/maps/:id", get(get_map))
        .with_state(app_state);

    let listener =
        match TcpListener::bind(format!("{}:{}", SETTINGS.api.host, SETTINGS.api.port)).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind api listener: {}", e);
                return;
            }
        };

    info!(
        "API Listening at http://{}:{}",
        SETTINGS.api.host, SETTINGS.api.port
    );

    if let Err(e) = axum::serve(listener, app).await {
        error!("Failed to start axum serve: {}", e);
    }
}
