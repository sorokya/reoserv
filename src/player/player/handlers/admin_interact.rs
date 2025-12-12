use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AdminInteractReportClientPacket, AdminInteractTellClientPacket},
        PacketAction, PacketFamily,
    },
};

use crate::{
    deep::{AdminInteractAddServerPacket, AdminInteractTakeClientPacket, DialogLine, LookupType},
    DROP_DB, ITEM_DB, NPC_DB, SETTINGS,
};

use super::super::Player;

impl Player {
    fn admin_interact_report(&mut self, reader: EoReader) {
        let report = match AdminInteractReportClientPacket::deserialize(&reader) {
            Ok(report) => report,
            Err(e) => {
                error!("Error deserializing AdminInteractReportClientPacket {}", e);
                return;
            }
        };

        let world = self.world.clone();
        let player_id = self.id;

        tokio::spawn(async move {
            world.report_player(player_id, report.reportee, report.message);
        });
    }

    fn admin_interact_tell(&mut self, reader: EoReader) {
        let tell = match AdminInteractTellClientPacket::deserialize(&reader) {
            Ok(tell) => tell,
            Err(e) => {
                error!("Error deserializing AdminInteractTellClientPacket {}", e);
                return;
            }
        };

        let world = self.world.to_owned();
        let player_id = self.id;

        tokio::spawn(async move {
            world.send_admin_message(player_id, tell.message);
        });
    }

    fn admin_interact_take(&mut self, reader: EoReader) {
        if !SETTINGS.world.info_reveals_drops {
            return;
        }

        let take = match AdminInteractTakeClientPacket::deserialize(&reader) {
            Ok(take) => take,
            Err(e) => {
                error!("Error deserializing AdminInteractTakeClientPacket: {}", e);
                return;
            }
        };

        match take.lookup_type {
            LookupType::Item => self.lookup_item(take.id),
            LookupType::Npc => self.lookup_npc(take.id),
            _ => {}
        }
    }

    fn lookup_item(&mut self, id: i32) {
        let map = match &self.map {
            Some(map) => map.clone(),
            None => return,
        };

        let mut lines = Vec::new();

        for npc in DROP_DB.npcs.iter() {
            if let Some(drop) = npc
                .drops
                .iter()
                .find(|drop| drop.item_id == id && drop.min_amount > 0 && drop.max_amount > 0)
            {
                let npc_name = match NPC_DB.npcs.get(npc.npc_id as usize - 1) {
                    Some(npc) => npc.name.to_owned(),
                    None => continue,
                };

                lines.push(DialogLine {
                    left: npc_name,
                    right: format!("{:.2}%", drop.rate as f32 / 64_000. * 100.),
                });
            }
        }

        if !lines.is_empty() {
            lines.insert(
                0,
                DialogLine {
                    left: "Drops:".to_string(),
                    right: String::default(),
                },
            );

            lines.insert(
                0,
                DialogLine {
                    left: ' '.to_string(),
                    right: String::default(),
                },
            );

            let player_id = self.id;

            tokio::spawn(async move {
                let player = match map.get_character(player_id).await.expect("Failed to get character. Timeout") {
                    Some(character) => match &character.player {
                        Some(player) => player.to_owned(),
                        None => return,
                    },
                    None => return,
                };

                player.send(
                    PacketAction::Add,
                    PacketFamily::AdminInteract,
                    &AdminInteractAddServerPacket { lines },
                );
            });
        }
    }

    fn lookup_npc(&mut self, id: i32) {
        let map = match &self.map {
            Some(map) => map.clone(),
            None => return,
        };

        let mut lines = Vec::new();

        let npc = match DROP_DB.npcs.iter().find(|npc| npc.npc_id == id) {
            Some(npc) => npc,
            None => return,
        };

        for drop in npc.drops.iter() {
            if drop.min_amount > 0 && drop.max_amount > 0 {
                let item_name = match ITEM_DB.items.get(drop.item_id as usize - 1) {
                    Some(item) => item.name.to_owned(),
                    None => continue,
                };

                lines.push(DialogLine {
                    left: item_name,
                    right: format!("{:.2}%", drop.rate as f32 / 64_000. * 100.),
                });
            }
        }

        if !lines.is_empty() {
            lines.insert(
                0,
                DialogLine {
                    left: "Drops:".to_string(),
                    right: String::default(),
                },
            );

            lines.insert(
                0,
                DialogLine {
                    left: ' '.to_string(),
                    right: String::default(),
                },
            );

            let player_id = self.id;

            tokio::spawn(async move {
                let player = match map.get_character(player_id).await.expect("Failed to get character. Timeout") {
                    Some(character) => match &character.player {
                        Some(player) => player.to_owned(),
                        None => return,
                    },
                    None => return,
                };

                player.send(
                    PacketAction::Add,
                    PacketFamily::AdminInteract,
                    &AdminInteractAddServerPacket { lines },
                );
            });
        }
    }

    pub fn handle_admin_interact(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Report => self.admin_interact_report(reader),
            PacketAction::Tell => self.admin_interact_tell(reader),
            PacketAction::Take => self.admin_interact_take(reader),
            _ => error!("Unhandled packet AdminInteract_{:?}", action),
        }
    }
}
