use eo::{
    data::{EOShort, EOInt},
    net::packets::{server::{account, character, login}, client},
};
use mysql_async::Pool;
use tokio::sync::{mpsc, oneshot};

use crate::player::PlayerHandle;

use super::{world::World, Command};

#[derive(Debug, Clone)]
pub struct WorldHandle {
    tx: mpsc::UnboundedSender<Command>,
    pub is_alive: bool,
}

impl WorldHandle {
    pub fn new(pool: Pool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let world = World::new(rx, pool);
        tokio::task::Builder::new()
            .name("run_world")
            .spawn(run_world(world));

        Self { tx, is_alive: true }
    }

    pub async fn start_ping_timer(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::StartPingTimer { respond_to: tx });
        rx.await.unwrap();
    }

    pub async fn get_player_count(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerCount { respond_to: tx });
        Ok(rx.await.unwrap())
    }

    pub async fn get_next_player_id(
        &self,
    ) -> Result<EOShort, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNextPlayerId { respond_to: tx });
        Ok(rx.await.unwrap())
    }

    pub async fn request_account_creation(
        &self,
        name: String,
        player: PlayerHandle,
    ) -> Result<account::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::RequestAccountCreation {
            name,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn create_account(
        &self,
        details: client::account::Create,
        register_ip: String,
    ) -> Result<account::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CreateAccount {
            details,
            register_ip,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn login(
        &mut self,
        name: String,
        password: String,
    ) -> Result<(login::Reply, EOShort), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Login {
            name,
            password,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn request_character_creation(
        &self,
        player: PlayerHandle,
    ) -> Result<character::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::RequestCharacterCreation {
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn create_character(
        &self,
        details: client::character::Create,
        player: PlayerHandle,
    ) -> Result<character::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CreateCharacter {
            details,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn request_character_deletion(
        &self,
        character_id: EOInt,
        player: PlayerHandle,
    ) -> Result<character::Player, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::RequestCharacterDeletion {
            character_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn delete_character(
        &self,
        session_id: EOShort,
        character_id: EOInt,
        player: PlayerHandle,
    ) -> Result<character::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::DeleteCharacter {
            session_id,
            character_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn add_player(
        &mut self,
        player_id: EOShort,
        player: PlayerHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddPlayer {
            player_id,
            player,
            respond_to: tx,
        });
        rx.await.unwrap();
        Ok(())
    }

    pub async fn drop_player(
        &mut self,
        player_id: EOShort,
        account_id: EOShort,
        character_id: EOShort,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::DropPlayer {
            respond_to: tx,
            player_id,
            account_id,
            character_id,
        });
        rx.await.unwrap();
        Ok(())
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
