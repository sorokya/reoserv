use axum::{
    routing::{get, post},
    Router,
};
use mysql_async::Pool;
use tokio::net::TcpListener;

use crate::{
    api::{
        routes::{login, root, user},
        AppState,
    },
    SETTINGS,
};

pub async fn run_api(pool: Pool) {
    let app_state = AppState { pool };

    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login))
        .route("/user", get(user))
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
