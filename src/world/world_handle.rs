use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};

use super::{world::World, Command};

#[derive(Debug)]
pub struct WorldHandle {
    tx: mpsc::UnboundedSender<Command>,
    pub is_alive: bool,
}

impl WorldHandle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let world = World::new(rx);
        tokio::task::Builder::new()
            .name("run_world")
            .spawn(run_world(world));

        Self { tx, is_alive: true }
    }

    pub async fn start_listener(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::StartListener { respond_to: tx });
        rx.await.unwrap();
    }

    pub async fn accept_connection(&self, world_handle: Arc<Mutex<WorldHandle>>) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AcceptConnection {
            respond_to: tx,
            world_handle,
        });
        rx.await.unwrap();
    }

    pub async fn load_maps(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::LoadMapFiles { respond_to: tx });
        rx.await.unwrap();
    }

    pub async fn load_pubs(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::LoadPubFiles { respond_to: tx });
        rx.await.unwrap();
    }
}

async fn run_world(mut world: World) {
    loop {
        if let Some(command) = world.rx.recv().await {
            world.handle_command(command).await;
        }
    }
}
