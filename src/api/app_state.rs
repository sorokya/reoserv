use axum::extract::FromRef;
use mysql_async::Pool;

use crate::world::WorldHandle;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub world: WorldHandle,
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for WorldHandle {
    fn from_ref(state: &AppState) -> Self {
        state.world.clone()
    }
}
