use axum::{
    routing::{get, post},
    Router,
};
use mysql_async::Pool;
use tokio::net::TcpListener;

use crate::api::{
    routes::{login, root, user},
    AppState,
};

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
