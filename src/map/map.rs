use std::{cmp, collections::HashMap};

use chrono::{Duration, Utc};
use eo::{
    data::{EOChar, EOInt, EOShort, EOThree, Serializeable},
    protocol::{
        server::{avatar, door, emote, face, npc, players, range, talk, walk},
        CharacterMapInfo, Coords, Direction, Emote, NPCUpdateChat, NPCUpdatePos, NearbyInfo,
        PacketAction, PacketFamily, WarpAnimation,
    },
    pubs::{EmfFile, EnfNpc},
};
use rand::Rng;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

use crate::{character::Character, world::WorldHandle, SETTINGS};

use super::{
    get_warp_at, is_in_bounds, is_occupied,
    is_tile_walkable::{is_tile_walkable, is_tile_walkable_for_npc},
    Command, Item, Npc, NpcData,
};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    world: WorldHandle,
    file: EmfFile,
    file_size: EOInt,
    id: EOShort,
    items: Vec<Item>,
    npcs: HashMap<EOChar, Npc>,
    npc_data: HashMap<EOShort, NpcData>,
    characters: HashMap<EOShort, Character>,
}

impl Map {
    pub fn new(
        id: EOShort,
        file_size: EOInt,
        file: EmfFile,
        rx: UnboundedReceiver<Command>,
        world: WorldHandle,
    ) -> Self {
        Self {
            id,
            file_size,
            file,
            rx,
            world,
            items: Vec::new(),
            npcs: HashMap::new(),
            npc_data: HashMap::new(),
            characters: HashMap::new(),
        }
    }

    async fn emote(&self, target_player_id: EOShort, emote: Emote) {
        if let Some(target) = self.characters.get(&target_player_id) {
            let packet = emote::Player {
                player_id: target_player_id,
                emote,
            };
            let buf = packet.serialize();
            for character in self.characters.values() {
                if character.player_id.unwrap() != target_player_id
                    && character.is_in_range(target.coords)
                {
                    debug!("Send: {:?}", packet);
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Emote,
                        buf.clone(),
                    );
                }
            }
        }
    }

    async fn enter(
        &mut self,
        new_character: Box<Character>,
        warp_animation: Option<WarpAnimation>,
        respond_to: oneshot::Sender<()>,
    ) {
        let mut character_map_info = new_character.to_map_info();
        character_map_info.animation = warp_animation;

        // TODO: Look into queueing this packet? (e.g multiple people entering the map at once)
        let mut packet = players::Agree::default();
        packet.nearby.num_characters = 1;
        packet.nearby.characters.push(character_map_info);
        let buf = packet.serialize();
        for character in self.characters.values() {
            if new_character.is_in_range(character.coords) {
                debug!("Send: {:?}", packet);
                character.player.as_ref().unwrap().send(
                    PacketAction::Agree,
                    PacketFamily::Players,
                    buf.clone(),
                );
            }
        }
        self.characters
            .insert(new_character.player_id.unwrap(), *new_character);
        let _ = respond_to.send(());
    }

    async fn face(&mut self, target_player_id: EOShort, direction: Direction) {
        {
            let mut target = self.characters.get_mut(&target_player_id).unwrap();
            target.direction = direction;
        }

        let packet = face::Player {
            player_id: target_player_id,
            direction,
        };
        let buf = packet.serialize();
        let target = self.characters.get(&target_player_id).unwrap();
        for character in self.characters.values() {
            if target_player_id != character.player_id.unwrap()
                && target.is_in_range(character.coords)
            {
                debug!("Send: {:?}", packet);
                character.player.as_ref().unwrap().send(
                    PacketAction::Player,
                    PacketFamily::Face,
                    buf.clone(),
                );
            }
        }
    }

    async fn get_map_info(
        &self,
        player_ids: Vec<EOShort>,
        npc_indexes: Vec<EOChar>,
        respond_to: oneshot::Sender<range::Reply>,
    ) {
        let mut reply = range::Reply::default();
        if player_ids.len() > 0 {
            for player_id in player_ids {
                if let Some(character) = self.characters.get(&player_id) {
                    if !reply
                        .nearby
                        .characters
                        .iter()
                        .any(|c: &CharacterMapInfo| c.id == player_id)
                    {
                        reply.nearby.num_characters += 1;
                        reply.nearby.characters.push(character.to_map_info());
                    }
                }
            }
        }

        if npc_indexes.len() > 0 {
            for npc_index in npc_indexes {
                if let Some(npc) = self.npcs.get(&npc_index) {
                    if npc.alive {
                        reply.nearby.npcs.push(npc.to_map_info(&npc_index));
                    }
                }
            }
        }

        let _ = respond_to.send(reply);
    }

    async fn open_door(&self, target_player_id: EOShort, door_coords: Coords) {
        let target = self.characters.get(&target_player_id).unwrap();
        if target.is_in_range(door_coords) {
            let packet = door::Open {
                coords: door_coords,
            };
            let buf = packet.serialize();
            for character in self.characters.values() {
                if character.is_in_range(door_coords) {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Open,
                        PacketFamily::Door,
                        buf.clone(),
                    );
                }
            }
        }
    }

    async fn walk(
        &mut self,
        target_player_id: EOShort,
        _timestamp: EOThree,
        _coords: Coords,
        direction: Direction,
    ) {
        if let Some((target_previous_coords, target_coords, target_player)) = {
            if let Some(target) = self.characters.get_mut(&target_player_id) {
                let previous_coords = target.coords;
                let mut coords = target.coords;
                match direction {
                    Direction::Up => coords.y -= 1,
                    Direction::Down => coords.y += 1,
                    Direction::Left => coords.x -= 1,
                    Direction::Right => coords.x += 1,
                }
                target.direction = direction;

                let is_tile_walkable = target.admin_level as EOChar >= 1
                    || is_tile_walkable(coords, &self.file.spec_rows);
                if is_in_bounds(coords, self.file.width, self.file.height) && is_tile_walkable {
                    target.coords = coords;
                }

                Some((previous_coords, target.coords, target.player.clone()))
            } else {
                None
            }
        } {
            // TODO: Ghost timer check
            if let Some(warp) = get_warp_at(target_coords, &self.file.warp_rows) {
                // TODO: verify warp requirements
                if let Some(target) = self.characters.get_mut(&target_player_id) {
                    target.player.as_ref().unwrap().request_warp(
                        warp.map,
                        warp.coords,
                        target.map_id == warp.map,
                        None,
                    );
                }
            } else {
                let packet = {
                    let mut packet = walk::Reply::default();

                    // This helped some but still doesn't feel "perfect"
                    let see_distance = match direction {
                        Direction::Down => (SETTINGS.world.see_distance - 1) as f64,
                        Direction::Left => (SETTINGS.world.see_distance - 3) as f64,
                        Direction::Up => (SETTINGS.world.see_distance - 1) as f64,
                        Direction::Right => (SETTINGS.world.see_distance - 3) as f64,
                    };

                    for (player_id, character) in self.characters.iter() {
                        if *player_id != target_player_id
                            && character.is_in_range_distance(target_coords, see_distance)
                            && !character.is_in_range_distance(target_previous_coords, see_distance)
                        {
                            packet.player_ids.push(*player_id);
                        }
                    }
                    for item in self.items.iter() {
                        if item.is_in_range_distance(target_coords, see_distance)
                            && !item.is_in_range_distance(target_previous_coords, see_distance)
                        {
                            packet.items.push(item.to_item_map_info());
                        }
                    }
                    for (index, npc) in self.npcs.iter() {
                        if npc.is_in_range_distance(target_coords, see_distance)
                            && !npc.is_in_range_distance(target_previous_coords, see_distance)
                        {
                            packet.npc_indexes.push(*index);
                        }
                    }
                    packet
                };

                debug!("Send: {:?}", packet);
                target_player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Walk,
                    packet.serialize(),
                );
            }

            let walk_packet = walk::Player {
                player_id: target_player_id,
                direction,
                coords: target_coords,
            };
            let walk_packet_buf = walk_packet.serialize();
            for (player_id, character) in self.characters.iter() {
                if target_player_id != *player_id && character.is_in_range(target_coords) {
                    debug!("Send: {:?}", walk_packet);
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Walk,
                        walk_packet_buf.clone(),
                    );
                }
            }
        }
    }

    async fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        if let Some(target) = self.characters.get(&target_player_id) {
            let packet = talk::Player {
                player_id: target_player_id,
                message,
            };
            let buf = packet.serialize();
            for character in self.characters.values() {
                if target_player_id != character.player_id.unwrap()
                    && target.is_in_range(character.coords)
                {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Talk,
                        buf.clone(),
                    );
                }
            }
        }
    }

    async fn spawn_npcs(&mut self) {
        // TODO: test if this is actually how GameServer.exe works
        let now = chrono::Utc::now();

        if !self.file.npcs.is_empty() {
            if self.npcs.is_empty() {
                let mut npc_index: EOChar = 0;

                let dead_since = if SETTINGS.npcs.instant_spawn {
                    now - Duration::days(1)
                } else {
                    now
                };

                for (spawn_index, spawn) in self.file.npcs.iter().enumerate() {
                    // Only 20% of npcs in a group will speak
                    let num_of_chatters =
                        cmp::max(1, (spawn.amount as f64 * 0.2).floor() as EOChar);
                    let mut chatter_indexes: Vec<usize> =
                        Vec::with_capacity(num_of_chatters as usize);
                    let chatter_distribution = spawn.amount / num_of_chatters;
                    for i in 0..num_of_chatters {
                        chatter_indexes.push(((i * chatter_distribution) + npc_index) as usize);
                    }

                    // TODO: bounds check
                    for _ in 0..spawn.amount {
                        self.npcs.insert(
                            npc_index,
                            Npc::new(
                                spawn.id,
                                Coords::default(),
                                Direction::Down,
                                spawn_index,
                                dead_since,
                                dead_since,
                                chatter_indexes.contains(&(npc_index as usize)),
                                now,
                            ),
                        );
                        npc_index += 1;
                    }

                    self.npc_data.entry(spawn.id).or_insert({
                        let data_record = match self.world.get_npc(spawn.id).await {
                            Ok(npc) => Some(npc),
                            Err(e) => {
                                error!("Failed to load NPC {}", e);
                                None
                            }
                        };

                        if data_record.is_some() {
                            let drop_record = self.world.get_drop_record(spawn.id).await;
                            let talk_record = self.world.get_talk_record(spawn.id).await;
                            NpcData {
                                npc_record: data_record.unwrap(),
                                drop_record,
                                talk_record,
                            }
                        } else {
                            warn!("Map {} has NPC {} but no NPC record", self.id, spawn.id);
                            NpcData {
                                npc_record: EnfNpc::default(),
                                drop_record: None,
                                talk_record: None,
                            }
                        }
                    });
                }
            }

            // let mut rng = rand::thread_rng();
            for npc in self.npcs.values_mut() {
                let spawn = &self.file.npcs[npc.spawn_index];
                if !npc.alive
                    && now.timestamp() - npc.dead_since.timestamp() > spawn.spawn_time.into()
                {
                    npc.alive = true;
                    npc.coords = Coords {
                        x: spawn.x,
                        y: spawn.y,
                    };

                    // TODO: bounds check
                    // while !is_tile_walkable_for_npc(
                    //     npc.coords,
                    //     &self.file.spec_rows,
                    //     &self.file.warp_rows,
                    // ) {
                    //     npc.coords.x += cmp::max(rng.gen_range(-1..=1), 0) as EOChar;
                    //     npc.coords.y += cmp::max(rng.gen_range(-1..=1), 0) as EOChar;
                    // }

                    npc.direction = if spawn.spawn_type == 7 {
                        Direction::from_char(spawn.spawn_type & 0x03).unwrap()
                    } else {
                        match rand::random::<u8>() % 4 {
                            0 => Direction::Down,
                            1 => Direction::Left,
                            2 => Direction::Up,
                            3 => Direction::Right,
                            _ => unreachable!(),
                        }
                    };
                }
            }
        }
    }

    fn act_npcs(&mut self) {
        if SETTINGS.npcs.freeze_on_empty_map && self.characters.is_empty() {
            return;
        }

        let now = Utc::now();

        let mut rng = rand::thread_rng();

        // TODO: attacks

        // get occupied tiles of all characters and npcs
        let mut occupied_tiles = Vec::new();
        for character in self.characters.values() {
            occupied_tiles.push(character.coords);
        }
        for npc in self.npcs.values() {
            occupied_tiles.push(npc.coords);
        }

        let mut position_updates: Vec<NPCUpdatePos> = Vec::with_capacity(self.npcs.len());
        let mut talk_updates: Vec<NPCUpdateChat> = Vec::with_capacity(self.npcs.len());

        for (index, npc) in &mut self.npcs {
            let spawn = &self.file.npcs[npc.spawn_index];
            let act_rate = match spawn.spawn_type {
                0 => SETTINGS.npcs.speed_0,
                1 => SETTINGS.npcs.speed_1,
                2 => SETTINGS.npcs.speed_2,
                3 => SETTINGS.npcs.speed_3,
                4 => SETTINGS.npcs.speed_4,
                5 => SETTINGS.npcs.speed_5,
                6 => SETTINGS.npcs.speed_6,
                7 => 0,
                _ => unreachable!("Invalid spawn type {} for NPC {}", spawn.spawn_type, npc.id),
            };

            let act_delta = now - npc.last_act;
            let walk_idle_for_ms = if let Some(walk_idle_for) = npc.walk_idle_for {
                walk_idle_for.num_milliseconds()
            } else {
                0
            };
            if npc.alive
                && act_rate > 0
                && act_delta >= Duration::milliseconds(act_rate as i64 + walk_idle_for_ms)
            {
                // TODO: attack

                let action = rng.gen_range(1..=10);
                if (7..=9).contains(&action) {
                    npc.direction = Direction::from_char(rng.gen_range(0..=3)).unwrap();
                }

                if action != 10 {
                    let new_coords = match npc.direction {
                        Direction::Down => {
                            if npc.coords.y >= self.file.height {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x,
                                    y: npc.coords.y + 1,
                                }
                            }
                        }
                        Direction::Left => {
                            if npc.coords.x == 0 {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x - 1,
                                    y: npc.coords.y,
                                }
                            }
                        }
                        Direction::Up => {
                            if npc.coords.y == 0 {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x,
                                    y: npc.coords.y - 1,
                                }
                            }
                        }
                        Direction::Right => {
                            if npc.coords.x >= self.file.width {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x + 1,
                                    y: npc.coords.y,
                                }
                            }
                        }
                    };

                    if !is_occupied(new_coords, &occupied_tiles)
                        && is_tile_walkable_for_npc(
                            new_coords,
                            &self.file.spec_rows,
                            &self.file.warp_rows,
                        )
                    {
                        // TODO: Fix if multiple npcs or players are on the same tile
                        occupied_tiles.retain(|coords| *coords != npc.coords);
                        npc.coords = new_coords;
                        position_updates.push(NPCUpdatePos {
                            npc_index: *index,
                            coords: npc.coords,
                            direction: npc.direction,
                        });
                        occupied_tiles.push(new_coords);
                    }

                    npc.last_act = Utc::now();
                    npc.walk_idle_for = None;
                } else {
                    npc.walk_idle_for = Some(Duration::seconds(rng.gen_range(1..=4)));
                }
            }

            if let Some(npc_data) = self.npc_data.get(&npc.id) {
                if let Some(talk_record) = &npc_data.talk_record {
                    let talk_delta = now - npc.last_talk;
                    if npc.alive
                        && npc.does_talk
                        && talk_delta >= Duration::milliseconds(SETTINGS.npcs.talk_rate as i64)
                    {
                        let roll = rng.gen_range(0..=100);
                        if roll <= talk_record.rate {
                            let message_index = rng.gen_range(0..talk_record.messages.len());
                            talk_updates.push(NPCUpdateChat {
                                npc_index: *index,
                                message_length: talk_record.messages[message_index].len() as EOChar,
                                message: talk_record.messages[message_index].to_string(),
                            })
                        }
                        npc.last_talk = now;
                    }
                }
            }
        }

        if !position_updates.is_empty() || !talk_updates.is_empty() {
            for character in self.characters.values() {
                // TODO: might also need to check NPCs previous position..

                let in_range_npc_indexes: Vec<EOChar> = self
                    .npcs
                    .iter()
                    .filter(|(_, n)| n.is_in_range(character.coords))
                    .map(|(i, _)| i)
                    .cloned()
                    .collect();

                let position_updates_in_rage: Vec<NPCUpdatePos> = position_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                let talk_updates_in_range: Vec<NPCUpdateChat> = talk_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                if !position_updates_in_rage.is_empty() || !talk_updates_in_range.is_empty() {
                    let packet = npc::Player {
                        pos: position_updates_in_rage,
                        attack: Vec::new(),
                        chat: talk_updates_in_range,
                    };

                    debug!("Send: {:?}", packet);
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Npc,
                        packet.serialize(),
                    );
                }
            }
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Emote {
                target_player_id,
                emote,
            } => self.emote(target_player_id, emote).await,
            Command::Enter {
                character,
                warp_animation,
                respond_to,
            } => self.enter(character, warp_animation, respond_to).await,
            Command::Face {
                target_player_id,
                direction,
            } => self.face(target_player_id, direction).await,
            Command::GetCharacter {
                player_id,
                respond_to,
            } => {
                if let Some(character) = self.characters.get(&player_id) {
                    let _ = respond_to.send(Some(Box::new(character.to_owned())));
                } else {
                    let _ = respond_to.send(None);
                }
            }
            Command::GetDimensions { respond_to } => {
                let _ = respond_to.send((self.file.width, self.file.height));
            }
            Command::GetMapInfo {
                player_ids,
                npc_indexes,
                respond_to,
            } => self.get_map_info(player_ids, npc_indexes, respond_to).await,
            Command::GetNearbyInfo {
                target_player_id,
                respond_to,
            } => {
                let target = self.characters.get(&target_player_id).unwrap();
                let mut nearby_items = Vec::new();
                let mut nearby_npcs = Vec::new();
                let mut nearby_characters = Vec::new();
                for item in self.items.iter() {
                    if target.is_in_range(item.coords) {
                        nearby_items.push(item.to_item_map_info());
                    }
                }
                for (index, npc) in self.npcs.iter() {
                    if npc.alive && target.is_in_range(npc.coords) {
                        nearby_npcs.push(npc.to_map_info(index));
                    }
                }
                for character in self.characters.values() {
                    if target_player_id == character.player_id.unwrap()
                        || target.is_in_range(character.coords)
                    {
                        nearby_characters.push(character.to_map_info());
                    }
                }
                let _ = respond_to.send(NearbyInfo {
                    num_characters: nearby_characters.len() as EOChar,
                    items: nearby_items,
                    npcs: nearby_npcs,
                    characters: nearby_characters,
                });
            }
            Command::GetRidAndSize { respond_to } => {
                let _ = respond_to.send((self.file.rid, self.file_size));
            }
            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => {
                let target = self.characters.remove(&target_player_id).unwrap();
                let packet = avatar::Remove {
                    player_id: target_player_id,
                    animation: warp_animation,
                };
                let buf = packet.serialize();
                for character in self.characters.values() {
                    if target.is_in_range(character.coords) {
                        debug!("Send: {:?}", packet);
                        character.player.as_ref().unwrap().send(
                            PacketAction::Remove,
                            PacketFamily::Avatar,
                            buf.clone(),
                        );
                    }
                }
                let _ = respond_to.send(target);
            }
            Command::OpenDoor {
                target_player_id,
                door_coords,
            } => self.open_door(target_player_id, door_coords).await,
            Command::SendChatMessage {
                target_player_id,
                message,
            } => self.send_chat_message(target_player_id, message).await,
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            }
            Command::SpawnNpcs => self.spawn_npcs().await,
            Command::ActNpcs => self.act_npcs(),
            Command::Walk {
                target_player_id,
                timestamp,
                coords,
                direction,
            } => {
                self.walk(target_player_id, timestamp, coords, direction)
                    .await
            }
        }
    }
}
