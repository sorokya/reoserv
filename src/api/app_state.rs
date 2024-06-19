use axum::extract::FromRef;
use mysql_async::Pool;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
