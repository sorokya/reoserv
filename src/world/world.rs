use crate::{
    connection_log::ConnectionLog, errors::DataNotFoundError, map::MapHandle, player::PlayerHandle,
    scripts::ScriptsHandle,
};

use super::{load_maps::load_maps, Command, Party};
use mysql_async::Pool;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub struct World {
    pub rx: UnboundedReceiver<Command>,
    scripts: Option<ScriptsHandle>,
    players: HashMap<i32, PlayerHandle>,
    accounts: Vec<i32>,
    pending_logins: Vec<i32>,
    characters: HashMap<String, i32>,
    guilds: HashMap<String, Vec<i32>>,
    pool: Pool,
    maps: Option<HashMap<i32, MapHandle>>,
    parties: Vec<Party>,
    npc_act_ticks: i32,
    item_spawn_ticks: i32,
    player_recover_ticks: i32,
    npc_recover_ticks: i32,
    quake_ticks: i32,
    spike_ticks: i32,
    drain_ticks: i32,
    second_ticks: i32,
    auto_pickup_ticks: i32,
    global_locked: bool,
    connection_log: ConnectionLog,
}

mod add_player;
mod admin;
mod chat;
mod disband_guild;
mod drop_player;
mod find_player;
mod get_character_by_name;
mod get_next_player_id;
mod party;
mod reload_map;
mod request_player_list;
mod request_player_name_list;
mod save;
mod shutdown;
mod tick;

impl World {
    pub fn new(rx: UnboundedReceiver<Command>, pool: Pool) -> Self {
        Self {
            rx,
            pool,
            scripts: None,
            players: HashMap::new(),
            accounts: Vec::new(),
            pending_logins: Vec::new(),
            characters: HashMap::new(),
            guilds: HashMap::new(),
            maps: None,
            parties: Vec::new(),
            npc_act_ticks: 0,
            item_spawn_ticks: 0,
            player_recover_ticks: 0,
            npc_recover_ticks: 0,
            quake_ticks: 0,
            spike_ticks: 0,
            drain_ticks: 0,
            second_ticks: 0,
            auto_pickup_ticks: 0,
            global_locked: false,
            connection_log: ConnectionLog::new(),
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::AcceptPartyRequest {
                player_id,
                target_player_id,
                request_type,
            } => {
                self.accept_party_request(player_id, target_player_id, request_type)
                    .await
            }

            Command::AddConnection { ip, respond_to } => {
                self.connection_log.add_connection(&ip);
                let _ = respond_to.send(());
            }

            Command::AddLoggedInAccount { account_id } => {
                self.pending_logins.retain(|id| *id != account_id);
                self.accounts.push(account_id);
            }

            Command::AddPendingLogin { account_id } => {
                self.pending_logins.push(account_id);
            }

            Command::RemovePendingLogin { account_id } => {
                self.pending_logins.retain(|id| *id != account_id);
            }

            Command::AddCharacter {
                player_id,
                name,
                guild_tag,
            } => {
                self.characters.insert(name, player_id);
                if let Some(guild_tag) = guild_tag {
                    match self.guilds.get_mut(&guild_tag) {
                        Some(guild) => {
                            guild.push(player_id);
                        }
                        None => {
                            self.guilds.insert(guild_tag, vec![player_id]);
                        }
                    }
                }
            }

            Command::AddGuildMember {
                player_id,
                guild_tag,
            } => match self.guilds.get_mut(&guild_tag) {
                Some(guild) => {
                    guild.push(player_id);
                }
                None => {
                    self.guilds.insert(guild_tag, vec![player_id]);
                }
            },

            Command::AddPlayer {
                respond_to,
                player_id,
                player,
            } => self.add_player(player_id, player, respond_to),

            Command::BroadcastAdminMessage { name, message } => {
                self.broadcast_admin_message(&name, &message).await
            }

            Command::BroadcastAnnouncement { name, message } => {
                self.broadcast_announcement(&name, &message).await
            }

            Command::BroadcastGlobalMessage {
                player_id: target_player_id,
                name,
                message,
            } => {
                self.broadcast_global_message(target_player_id, &name, &message)
                    .await
            }

            Command::BroadcastPartyMessage { player_id, message } => {
                self.broadcast_party_message(player_id, message);
            }

            Command::BroadcastGuildMessage {
                player_id,
                guild_tag,
                name,
                message,
            } => {
                self.broadcast_guild_message(player_id, guild_tag, name, message);
            }

            Command::DisbandGuild { guild_tag } => self.disband_guild(guild_tag),

            Command::DropPlayer {
                player_id,
                ip,
                account_id,
                character_name,
                guild_tag,
                respond_to,
            } => self.drop_player(
                player_id,
                ip,
                account_id,
                &character_name,
                guild_tag,
                respond_to,
            ),

            Command::GetCharacterByName { name, respond_to } => {
                let _ = respond_to.send(self.get_character_by_name(&name).await);
            }

            Command::GetConnectionCount { respond_to } => {
                let _ = respond_to.send(self.connection_log.len());
            }

            Command::GetIpConnectionCount { ip, respond_to } => {
                let _ = respond_to.send(self.connection_log.get_num_of_connections(&ip));
            }

            Command::GetIpLastConnect { ip, respond_to } => {
                let _ = respond_to.send(self.connection_log.get_last_connect(&ip));
            }

            Command::GetMap { map_id, respond_to } => {
                let maps = self.maps.as_ref().expect("maps not loaded");
                match maps.get(&map_id) {
                    Some(map) => {
                        let _ = respond_to.send(Ok(map.to_owned()));
                    }
                    None => {
                        warn!("Map not found: {}", map_id);
                        let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                            "Map".to_string(),
                            map_id,
                        ))));
                    }
                }
            }

            Command::GetNextPlayerId { respond_to } => {
                let _ = respond_to.send(self.get_next_player_id(300));
            }

            Command::GetPlayer {
                player_id,
                respond_to,
            } => {
                let _ = respond_to.send(self.players.get(&player_id).cloned());
            }

            Command::GetPlayerCount { respond_to } => {
                let _ = respond_to.send(self.characters.len() as i32);
            }

            Command::GetPlayerParty {
                player_id,
                respond_to,
            } => {
                let _ = respond_to.send(self.get_player_party(player_id));
            }

            Command::IsLoggedIn {
                account_id,
                respond_to,
            } => {
                let _ = respond_to.send(
                    self.accounts.contains(&account_id)
                        || self.pending_logins.contains(&account_id),
                );
            }

            Command::LoadMapFiles { world, respond_to } => {
                match load_maps(self.pool.to_owned(), world).await {
                    Ok(maps) => {
                        self.maps = Some(maps);
                        let _ = respond_to.send(());
                    }
                    Err(err) => {
                        warn!("Failed to load maps: {}", err);
                        let _ = respond_to.send(());
                    }
                }
            }

            Command::RemoveGuildMember {
                player_id,
                guild_tag,
            } => {
                let remaining = match self.guilds.get_mut(&guild_tag) {
                    Some(guild) => {
                        guild.retain(|&id| id != player_id);
                        guild.len()
                    }
                    None => 0,
                };

                if remaining == 0 {
                    self.guilds.remove(&guild_tag);
                }
            }

            Command::RemovePartyMember {
                player_id,
                target_player_id,
            } => self.remove_party_member(player_id, target_player_id),

            Command::ReportPlayer {
                player_id,
                reportee_name,
                message,
            } => self.report_player(player_id, reportee_name, message).await,

            Command::RequestPartyList { player_id } => self.refresh_party(player_id).await,

            Command::ReloadMap { map_id } => self.reload_map(map_id).await,

            Command::Save => self.save().await,

            Command::SendAdminMessage { player_id, message } => {
                self.send_admin_message(player_id, message).await
            }

            Command::SendPrivateMessage {
                player_id,
                to,
                message,
            } => self.send_private_message(player_id, &to, &message).await,

            Command::SetScripts(scripts) => self.scripts = Some(scripts),

            Command::ShowCaptcha {
                victim_name,
                experience,
            } => self.show_captcha(victim_name, experience),

            Command::Shutdown { respond_to } => self.shutdown(respond_to).await,

            Command::Tick => {
                self.tick().await;
            }

            Command::UpdatePartyHP {
                player_id,
                hp_percentage,
            } => self.update_party_hp(player_id, hp_percentage),

            Command::UpdatePartyExp {
                player_id,
                exp_gains,
            } => self.update_party_exp(player_id, exp_gains),

            Command::BanPlayer {
                victim_name,
                admin_name,
                duration,
                silent,
            } => {
                self.ban_player(victim_name, admin_name, duration, silent)
                    .await
            }

            Command::FindPlayer { player_id, name } => self.find_player(player_id, name),

            Command::FreePlayer { victim_name } => self.free_player(victim_name),

            Command::FreezePlayer {
                victim_name,
                admin_name,
            } => self.freeze_player(victim_name, admin_name),

            Command::JailPlayer {
                victim_name,
                admin_name,
            } => self.jail_player(victim_name, admin_name),

            Command::KickPlayer {
                victim_name,
                admin_name,
                silent,
            } => self.kick_player(victim_name, admin_name, silent),

            Command::MutePlayer {
                victim_name,
                admin_name,
            } => self.mute_player(victim_name, admin_name),

            Command::Quake { magnitude } => self.quake(magnitude),

            Command::RequestPlayerInfo {
                player_id,
                victim_name,
            } => self.request_player_info(player_id, victim_name),

            Command::RequestPlayerInventory {
                player_id,
                victim_name,
            } => self.request_player_inventory(player_id, victim_name),

            Command::ToggleGlobal { admin_name } => self.toggle_global(admin_name),

            Command::UnfreezePlayer {
                victim_name,
                admin_name,
            } => self.unfreeze_player(victim_name, admin_name),
            Command::RequestPlayerList { player_id } => self.request_player_list(player_id),
            Command::RequestPlayerNameList { player_id } => {
                self.request_player_name_list(player_id)
            }
        }
    }
}
