use std::{collections::HashMap, sync::Arc};

use eo::{
    data::{map::MapFile, EOShort, Serializeable},
    net::{ItemMapInfo, NearbyInfo, NpcMapInfo},
};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::player::PlayerHandle;

use super::{Command, Item, NPC};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: MapFile,
    items: Arc<Mutex<Vec<Item>>>,
    npcs: Arc<Mutex<Vec<NPC>>>,
    players: Arc<Mutex<HashMap<EOShort, PlayerHandle>>>,
}

impl Map {
    pub fn new(file: MapFile, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file,
            rx,
            items: Arc::new(Mutex::new(Vec::new())),
            npcs: Arc::new(Mutex::new(Vec::new())),
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Enter(player_id, player) => {
                // let coords = player.get_coords().await;
                let mut players = self.players.lock().await;
                players.insert(player_id, player);

                // TODO: send character map info to nearby players
                // let players = players;
                // for player in players.values() {
                //     player.send_if_in_range()
                // }
            }
            Command::GetHashAndSize { respond_to } => {
                let _ = respond_to.send((self.file.hash, self.file.size));
            }
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            }
            Command::GetNearbyInfo {
                target_player_id,
                respond_to,
            } => {
                let players = self.players.lock().await;
                let target = players.get(&target_player_id).unwrap();
                let items = self.items.lock().await;
                let npcs = self.npcs.lock().await;
                let mut nearby_items = Vec::new();
                let mut nearby_npcs = Vec::new();
                let mut nearby_characters = Vec::new();
                for item in items.iter() {
                    if target.is_in_range(item.coords).await {
                        nearby_items.push(item.to_item_map_info());
                    }
                }
                for npc in npcs.iter() {
                    if target.is_in_range(npc.coords.to_coords()).await {
                        nearby_npcs.push(npc.to_npc_map_info());
                    }
                }
                for player in players.values() {
                    let player_id = player.get_player_id().await;
                    // TODO: don't unwrap
                    if target_player_id == player_id
                        || target.is_in_range(player.get_coords().await.unwrap()).await
                    {
                        nearby_characters.push(player.get_character_map_info().await.unwrap());
                    }
                }
                let _ = respond_to.send(NearbyInfo {
                    items: nearby_items,
                    npcs: nearby_npcs,
                    characters: nearby_characters,
                });
            }
        }
    }
}
