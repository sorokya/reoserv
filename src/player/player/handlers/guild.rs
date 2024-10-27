use chrono::NaiveDateTime;
use eolib::{
    data::{EoReader, EoSerialize},
    protocol::{
        net::{
            client::{
                GuildAcceptClientPacket, GuildAgreeClientPacket,
                GuildAgreeClientPacketInfoTypeData, GuildBuyClientPacket, GuildCreateClientPacket,
                GuildInfoType, GuildJunkClientPacket, GuildKickClientPacket, GuildOpenClientPacket,
                GuildPlayerClientPacket, GuildRankClientPacket, GuildRemoveClientPacket,
                GuildReportClientPacket, GuildRequestClientPacket, GuildTakeClientPacket,
                GuildTellClientPacket, GuildUseClientPacket,
            },
            server::{
                GuildAcceptServerPacket, GuildAgreeServerPacket, GuildMember,
                GuildRankServerPacket, GuildReply, GuildReplyServerPacket, GuildReportServerPacket,
                GuildSellServerPacket, GuildStaff, GuildTakeServerPacket, GuildTellServerPacket,
            },
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};
use futures::StreamExt;
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

use crate::{
    player::{
        player::guild::{
            guild_exists, validate_guild_description, validate_guild_name, validate_guild_rank,
            validate_guild_tag,
        },
        PlayerHandle,
    },
    utils::{capitalize, get_guild_ranks},
    NPC_DB, SETTINGS,
};

use super::super::Player;

impl Player {
    fn guild_open(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let open = match GuildOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing GuildOpenClientPacket: {}", e);
                    return;
                }
            };

            map.open_guild_master(self.id, open.npc_index);
        }
    }

    fn guild_request(&mut self, reader: EoReader) {
        let request = match GuildRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing GuildRequestClientPacket: {}", e);
                return;
            }
        };

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        match self.session_id {
            Some(session_id) => {
                if session_id != request.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let player_id = self.id;
        let pool = self.pool.clone();

        self.guild_create_members = Vec::with_capacity(SETTINGS.guild.min_players);

        tokio::spawn(async move {
            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => return,
            };

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            if !validate_guild_tag(&request.guild_tag) || !validate_guild_name(&request.guild_name)
            {
                player.send_guild_reply(GuildReply::NotApproved);
                return;
            }

            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return,
            };

            if npc_data.r#type != NpcType::Guild {
                return;
            }

            if character.guild_tag.is_some()
                || character.get_item_amount(1) < SETTINGS.guild.create_cost
            {
                return;
            }

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting SQL connection: {}", e);
                    return;
                }
            };

            if guild_exists(&mut conn, &request.guild_tag, &request.guild_name).await {
                player.send_guild_reply(GuildReply::Exists);
                return;
            }

            if SETTINGS.guild.min_players == 1 {
                player.send_guild_reply(GuildReply::CreateAddConfirm);
            } else {
                let player_count = map.get_player_count(|c| c.guild_tag.is_none()).await;
                if player_count < SETTINGS.guild.min_players {
                    player.send_guild_reply(GuildReply::NoCandidates);
                    return;
                }

                player.send_guild_reply(GuildReply::CreateBegin);
                map.send_guild_create_requests(
                    player_id,
                    format!(
                        "{} ({})",
                        capitalize(&character.name.to_lowercase()),
                        request.guild_tag.to_uppercase()
                    ),
                );
            }
        });
    }

    fn guild_accept(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let accept = match GuildAcceptClientPacket::deserialize(&reader) {
                Ok(accept) => accept,
                Err(e) => {
                    error!("Error deserializing GuildAcceptClientPacket: {}", e);
                    return;
                }
            };

            map.accept_guild_creation_request(self.id, accept.inviter_player_id);
        }
    }

    fn guild_create(&mut self, reader: EoReader) {
        let create = match GuildCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                error!("Error deserializing GuildCreateClientPacket: {}", e);
                return;
            }
        };

        if self.guild_create_members.len() + 1 < SETTINGS.guild.min_players {
            return;
        }

        match self.session_id {
            Some(session_id) => {
                if session_id != create.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let player_id = self.id;
        let pool = self.pool.clone();
        let guild_create_members = self.guild_create_members.clone();
        self.guild_create_members.clear();

        tokio::spawn(async move {
            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => return,
            };

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            if !validate_guild_tag(&create.guild_tag)
                || !validate_guild_name(&create.guild_name)
                || !validate_guild_description(&create.description)
            {
                player.send_guild_reply(GuildReply::NotApproved);
                return;
            }

            if character.guild_tag.is_some()
                || character.get_item_amount(1) < SETTINGS.guild.create_cost
            {
                return;
            }

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting SQL connection: {}", e);
                    return;
                }
            };

            if guild_exists(&mut conn, &create.guild_tag, &create.guild_name).await {
                player.send_guild_reply(GuildReply::Exists);
                return;
            }

            let guild_tag = create.guild_tag.to_uppercase();

            if let Err(e) = create_guild(
                &mut conn,
                &guild_tag,
                &create.guild_name,
                &create.description,
            )
            .await
            {
                error!("Error creating guild: {}", e);
                return;
            }

            map.finish_guild_creation(
                player_id,
                guild_create_members,
                guild_tag,
                create.guild_name,
            );
        });
    }

    fn guild_player(&mut self, reader: EoReader) {
        let packet = match GuildPlayerClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildPlayerClientPacket: {}", e);
                return;
            }
        };

        match self.session_id {
            Some(session_id) => {
                if session_id != packet.session_id {
                    return;
                }
            }
            None => return,
        }

        if let Some(map) = &self.map {
            map.request_to_join_guild(self.id, packet.guild_tag, packet.recruiter_name);
        }
    }

    fn guild_use(&mut self, reader: EoReader) {
        let packet = match GuildUseClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildUseClientPacket: {}", e);
                return;
            }
        };

        match self.interact_player_id {
            Some(id) => {
                if id != packet.player_id {
                    return;
                }
            }
            None => return,
        }

        self.interact_player_id = None;

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let recruiter_id = self.id;

        let pool = self.pool.clone();

        tokio::spawn(async move {
            let character = match map.get_character(recruiter_id).await {
                Some(character) => character,
                None => return,
            };

            let tag = match character.guild_tag {
                Some(ref tag) => tag,
                None => return,
            };

            if character.guild_rank.unwrap() > 1 {
                return;
            }

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let guild_bank = get_guild_bank(&mut conn, character.guild_tag.as_ref().unwrap()).await;
            if guild_bank < SETTINGS.guild.recruit_cost {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Guild,
                    &GuildReplyServerPacket {
                        reply_code: GuildReply::AccountLow,
                        reply_code_data: None,
                    },
                );
                return;
            }

            if let Err(e) =
                set_guild_bank(&mut conn, tag, guild_bank - SETTINGS.guild.recruit_cost).await
            {
                error!("Error setting guild bank: {}", e);
                return;
            }

            let guild_name = match get_guild_name(&mut conn, tag).await {
                Some(guild_name) => guild_name,
                None => return,
            };

            let rank_string = match get_new_member_guild_rank(&mut conn, tag).await {
                Some(rank_string) => rank_string,
                None => return,
            };

            map.join_guild(
                packet.player_id,
                recruiter_id,
                tag.to_owned(),
                guild_name,
                rank_string,
            );
        });
    }

    fn guild_kick(&mut self, reader: EoReader) {
        let packet = match GuildKickClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildKickClientPacket: {}", e);
                return;
            }
        };

        match self.session_id {
            Some(id) => {
                if id != packet.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let player_id = self.id;
        let world = self.world.clone();

        tokio::spawn(async move {
            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => return,
            };

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            if !character.is_guild_leader() {
                return;
            }

            match world
                .get_character_by_name(&packet.member_name)
                .await
            {
                Ok(member) => {
                    if member.guild_tag != character.guild_tag {
                        player.send_guild_reply(GuildReply::RemoveNotMember);
                        return;
                    }

                    if member.is_guild_leader() {
                        player.send_guild_reply(GuildReply::RemoveLeader);
                        return;
                    }

                    let member_map = match world.get_map(member.map_id).await {
                        Ok(map) => map,
                        Err(_) => {
                            error!("Error getting map {}", member.map_id);
                            return;
                        }
                    };

                    member_map.kick_from_guild(member.player_id.unwrap());

                    // TODO: Guild announce
                    // self.world.guild_announcement(guild_id, "Blah was kicked from the guild")

                    player.send_guild_reply(GuildReply::Removed);
                }
                Err(_) => {
                    // TODO: Offline kick
                    player.send_server_message("Offline kicking not currently supported");
                }
            }
        });
    }

    fn guild_take(&mut self, reader: EoReader) {
        let packet = match GuildTakeClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildTakeClientPacket: {}", e);
                return;
            }
        };

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        match self.session_id {
            Some(id) => {
                if id != packet.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let pool = self.pool.clone();
        let player_id = self.id;

        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return,
            };

            if npc_data.r#type != NpcType::Guild {
                return;
            }

            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => return,
            };

            if character.guild_tag.is_none() {
                return;
            }

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            match packet.info_type {
                GuildInfoType::Description => {
                    let description =
                        get_guild_description(&mut conn, character.guild_tag.as_ref().unwrap())
                            .await;

                    player.send(
                        PacketAction::Take,
                        PacketFamily::Guild,
                        &GuildTakeServerPacket { description },
                    );
                }
                GuildInfoType::Ranks => {
                    let ranks =
                        get_guild_ranks(&mut conn, character.guild_tag.as_ref().unwrap()).await;

                    player.send(
                        PacketAction::Rank,
                        PacketFamily::Guild,
                        &GuildRankServerPacket {
                            ranks: [
                                ranks[0].to_owned(),
                                ranks[1].to_owned(),
                                ranks[2].to_owned(),
                                ranks[3].to_owned(),
                                ranks[4].to_owned(),
                                ranks[5].to_owned(),
                                ranks[6].to_owned(),
                                ranks[7].to_owned(),
                                ranks[8].to_owned(),
                            ],
                        },
                    );
                }
                GuildInfoType::Bank => {
                    let gold_amount =
                        get_guild_bank(&mut conn, character.guild_tag.as_ref().unwrap()).await;

                    player.send(
                        PacketAction::Sell,
                        PacketFamily::Guild,
                        &GuildSellServerPacket { gold_amount },
                    );
                }
                _ => {}
            }
        });
    }

    fn guild_buy(&mut self, reader: EoReader) {
        let packet = match GuildBuyClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildBuyClientPacket: {}", e);
                return;
            }
        };

        if packet.gold_amount < SETTINGS.guild.min_deposit {
            return;
        }

        match self.session_id {
            Some(session_id) => {
                if session_id != packet.session_id {
                    return;
                }
            }
            None => {
                return;
            }
        };

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        map.deposit_guild_gold(self.id, npc_index, packet.gold_amount);
    }

    fn guild_agree(&mut self, reader: EoReader) {
        let packet = match GuildAgreeClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildAgreeClientPacket: {}", e);
                return;
            }
        };

        let info_type_data = match packet.info_type_data {
            Some(info_type_data) => info_type_data,
            None => return,
        };

        match self.session_id {
            Some(id) => {
                if id != packet.session_id {
                    return;
                }
            }
            None => return,
        }

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let pool = self.pool.clone();

        let player_id = self.id;

        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get SQL connection: {}", e);
                    return;
                }
            };

            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => {
                    if npc_data.r#type != NpcType::Guild {
                        return;
                    }
                }
                None => return,
            };

            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => {
                    return;
                }
            };

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            match character.guild_rank {
                Some(rank_index) => {
                    if rank_index > 1 {
                        return;
                    }
                }
                None => return,
            }

            match info_type_data {
                GuildAgreeClientPacketInfoTypeData::Description(description) => {
                    update_guild_description(
                        player,
                        character.guild_tag.as_ref().unwrap(),
                        description.description,
                        &mut conn,
                    )
                    .await
                }
                GuildAgreeClientPacketInfoTypeData::Ranks(ranks) => {
                    update_guild_ranks(
                        player,
                        character.guild_tag.as_ref().unwrap(),
                        ranks.ranks,
                        &mut conn,
                    )
                    .await
                }
            }
        });
    }

    fn guild_rank(&mut self, reader: EoReader) {
        let packet = match GuildRankClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildRankClientPacket: {}", e);
                return;
            }
        };

        if !(1..=9).contains(&packet.rank) {
            return;
        }

        match self.session_id {
            Some(id) => {
                if id != packet.session_id {
                    return;
                }
            }
            None => return,
        }

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.clone();

        let pool = self.pool.clone();

        let player_id = self.id;

        tokio::spawn(async move {
            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => {
                    if npc_data.r#type != NpcType::Guild {
                        return;
                    }
                }
                None => return,
            };

            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => {
                    return;
                }
            };

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            if !character.is_guild_leader() {
                return;
            }

            let guild_tag = match character.guild_tag {
                Some(ref guild_tag) => guild_tag,
                None => return,
            };

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let ranks = get_guild_ranks(&mut conn, guild_tag).await;
            let rank_str = match ranks.get(packet.rank as usize - 1) {
                Some(rank) => rank,
                None => return,
            };

            let target_character = match world
                .get_character_by_name(&packet.member_name)
                .await
            {
                Ok(character) => character,
                Err(_) => {
                    player.send_server_message("Offline rank updating not currently supported");
                    return;
                    // TODO: handle offline
                    /* self.assign_guild_rank_offline(guild_tag, member_name, rank, rank_str)
                        .await;
                    return; */
                }
            };

            let target_guild_tag = match target_character.guild_tag {
                Some(ref guild_tag) => guild_tag,
                None => return,
            };

            if guild_tag != target_guild_tag {
                player.send_guild_reply(GuildReply::RankingNotMember);
                return;
            }

            if target_character.is_guild_leader() {
                player.send_guild_reply(GuildReply::RankingLeader);
                return;
            }

            let map = match world.get_map(target_character.map_id).await {
                Ok(map) => map,
                Err(e) => {
                    error!("Error getting map: {}", e);
                    return;
                }
            };

            map.update_guild_rank(target_character.player_id.unwrap(), packet.rank, rank_str);

            player.send_guild_reply(GuildReply::Updated);
        });
    }

    fn guild_report(&mut self, reader: EoReader) {
        let report = match GuildReportClientPacket::deserialize(&reader) {
            Ok(report) => report,
            Err(e) => {
                error!("Error deserializing GuildReportClientPacket: {}", e);
                return;
            }
        };

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        match self.session_id {
            Some(id) => {
                if id != report.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let pool = self.pool.clone();
        let player_id = self.id;

        tokio::spawn(async move {
            let player = match map.get_character(player_id).await {
                Some(character) => match &character.player {
                    Some(player) => player.to_owned(),
                    None => return,
                },
                None => return,
            };

            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return,
            };

            if npc_data.r#type != NpcType::Guild {
                return;
            }

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let mut result = match conn
                .exec_iter(
                    "CALL GetGuildDetails(:guild_identity);",
                    params! {
                        "guild_identity" => &report.guild_identity,
                    },
                )
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    error!("Error getting guild details: {}", e);
                    return;
                }
            };

            let mut packet = GuildReportServerPacket::default();

            {
                let mut stream = match result.stream::<Row>().await {
                    Ok(Some(stream)) => stream,
                    Ok(None) => {
                        player.send_guild_reply(GuildReply::NotFound);
                        return;
                    }
                    Err(e) => {
                        error!("Error getting guild details: {}", e);
                        return;
                    }
                };

                let mut row = match stream.next().await {
                    Some(Ok(row)) => row,
                    Some(Err(e)) => {
                        error!("Error getting guild details: {}", e);
                        return;
                    }
                    None => {
                        error!("Error getting guild details: no rows returned");
                        return;
                    }
                };

                packet.tag = row.take("tag").unwrap();
                packet.name = row.take("name").unwrap();
                packet.description = row.take("description").unwrap();

                let created_at: NaiveDateTime = row.take("created_at").unwrap();
                packet.create_date = created_at.format("%Y-%m-%d").to_string();

                let bank: i32 = row.take("bank").unwrap();
                packet.wealth = if bank < 2000 {
                    "bankrupt".to_string()
                } else if bank < 10_000 {
                    "poor".to_string()
                } else if bank < 50_000 {
                    "normal".to_string()
                } else if bank < 100_000 {
                    "wealthy".to_string()
                } else {
                    "very wealthy".to_string()
                };
            }

            {
                let mut stream = match result.stream::<Row>().await {
                    Ok(Some(stream)) => stream,
                    Ok(None) => {
                        player.send_guild_reply(GuildReply::NotFound);
                        return;
                    }
                    Err(e) => {
                        error!("Error getting guild details: {}", e);
                        return;
                    }
                };

                let mut index = 0;
                while let Some(row) = stream.next().await {
                    let mut row = match row {
                        Ok(row) => row,
                        Err(e) => {
                            error!("Error getting guild details: {}", e);
                            return;
                        }
                    };

                    // Client won't display ranks less than 4 characters long
                    packet.ranks[index] =
                        format!("{:<4}", row.take::<String, &str>("rank").unwrap());

                    index += 1;
                }
            }

            {
                let mut stream = match result.stream::<Row>().await {
                    Ok(Some(stream)) => stream,
                    Ok(None) => {
                        player.send_guild_reply(GuildReply::NotFound);
                        return;
                    }
                    Err(e) => {
                        error!("Error getting guild details: {}", e);
                        return;
                    }
                };

                while let Some(row) = stream.next().await {
                    let mut row = match row {
                        Ok(row) => row,
                        Err(e) => {
                            error!("Error getting guild details: {}", e);
                            return;
                        }
                    };

                    packet.staff.push(GuildStaff {
                        rank: row.take("guild_rank").unwrap(),
                        name: row.take("name").unwrap(),
                    });
                }
            }

            player.send(PacketAction::Report, PacketFamily::Guild, &packet);
        });
    }

    fn guild_tell(&mut self, reader: EoReader) {
        let tell = match GuildTellClientPacket::deserialize(&reader) {
            Ok(tell) => tell,
            Err(e) => {
                error!("Error deserializing GuildTellClientPacket: {}", e);
                return;
            }
        };

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        match self.session_id {
            Some(id) => {
                if id != tell.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.clone(),
            None => return,
        };

        let pool = self.pool.clone();

        let player_id = self.id;

        tokio::spawn(async move {
            let player = match map.get_character(player_id).await {
                Some(character) => match &character.player {
                    Some(player) => player.to_owned(),
                    None => return,
                },
                None => return,
            };

            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return,
            };

            if npc_data.r#type != NpcType::Guild {
                return;
            }

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let members: Vec<GuildMember> = match conn
                .exec_map(
                    include_str!("../../../sql/get_guild_memberlist.sql"),
                    params! {
                        "guild_identity" => &tell.guild_identity,
                    },
                    |mut row: Row| GuildMember {
                        rank: row.take("guild_rank").unwrap(),
                        name: row.take("name").unwrap(),
                        rank_name: row.take("guild_rank_string").unwrap(),
                    },
                )
                .await
            {
                Ok(members) => members,
                Err(e) => {
                    error!("Error getting guild memberlist: {}", e);
                    return;
                }
            };

            if members.is_empty() {
                player.send_guild_reply(GuildReply::NotFound);
                return;
            }

            player.send(
                PacketAction::Tell,
                PacketFamily::Guild,
                &GuildTellServerPacket { members },
            );
        });
    }

    fn guild_remove(&mut self, reader: EoReader) {
        let remove = match GuildRemoveClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildRemoveClientPacket: {}", e);
                return;
            }
        };

        match self.session_id {
            Some(id) => {
                if id != remove.session_id {
                    return;
                }
            }
            None => return,
        }

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let player_id = self.id;

        let pool = self.pool.clone();

        tokio::spawn(async move {
            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => return,
            };

            let player = match &character.player {
                Some(player) => player,
                None => return,
            };

            let guild_tag = match character.guild_tag {
                Some(ref tag) => tag,
                None => return,
            };

            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            if character.guild_rank == Some(1) {
                let leader_count = match conn
                    .exec_map(
                        include_str!("../../../sql/get_count_guild_leader.sql"),
                        params! {
                            "guild_tag" => guild_tag,
                        },
                        |mut row: Row| row.take::<i32, usize>(0).unwrap(),
                    )
                    .await
                {
                    Ok(leader_counts) => match leader_counts.first() {
                        Some(leader_count) => *leader_count,
                        None => 0,
                    },
                    Err(e) => {
                        error!("Error getting leader count: {}", e);
                        return;
                    }
                };

                if leader_count == 1 {
                    player.send_server_message("You are the last leader and cannot leave the guild. You must promote someone else to leader first.");

                    // This is dumb but it tricks the v28 client into keeping you in your guild
                    player.send(
                        PacketAction::Agree,
                        PacketFamily::Guild,
                        &GuildAgreeServerPacket {
                            recruiter_id: player_id,
                            guild_tag: guild_tag.to_owned(),
                            guild_name: character.guild_name.unwrap().clone(),
                            rank_name: character.guild_rank_string.unwrap().clone(),
                        },
                    );

                    player.send(
                        PacketAction::Accept,
                        PacketFamily::Guild,
                        &GuildAcceptServerPacket { rank: 1 },
                    );

                    return;
                }
            }

            map.leave_guild(player_id);
        });
    }

    fn guild_junk(&mut self, reader: EoReader) {
        let junk = match GuildJunkClientPacket::deserialize(&reader) {
            Ok(packet) => packet,
            Err(e) => {
                error!("Error deserializing GuildJunkClientPacket: {}", e);
                return;
            }
        };

        match self.session_id {
            Some(id) => {
                if id != junk.session_id {
                    return;
                }
            }
            None => return,
        }

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        let map = match &self.map {
            Some(map) => map.to_owned(),
            None => return,
        };

        let world = self.world.clone();
        let player_id = self.id;

        tokio::spawn(async move {
            let npc_id = match map.get_npc_id_for_index(npc_index).await {
                Some(npc_id) => npc_id,
                None => return,
            };

            match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => {
                    if npc_data.r#type != NpcType::Guild {
                        return;
                    }
                }
                None => return,
            };

            let character = match map.get_character(player_id).await {
                Some(character) => character,
                None => {
                    return;
                }
            };

            if !character.is_guild_leader() {
                return;
            }

            let guild_tag = match character.guild_tag {
                Some(ref guild_tag) => guild_tag,
                None => return,
            };

            world.disband_guild(guild_tag.to_owned());
        });
    }

    pub fn handle_guild(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Open => self.guild_open(reader),
            PacketAction::Request => self.guild_request(reader),
            PacketAction::Accept => self.guild_accept(reader),
            PacketAction::Create => self.guild_create(reader),
            PacketAction::Player => self.guild_player(reader),
            PacketAction::Use => self.guild_use(reader),
            PacketAction::Kick => self.guild_kick(reader),
            PacketAction::Take => self.guild_take(reader),
            PacketAction::Buy => self.guild_buy(reader),
            PacketAction::Agree => self.guild_agree(reader),
            PacketAction::Rank => self.guild_rank(reader),
            PacketAction::Report => self.guild_report(reader),
            PacketAction::Tell => self.guild_tell(reader),
            PacketAction::Remove => self.guild_remove(reader),
            PacketAction::Junk => self.guild_junk(reader),
            _ => error!("Unhandled packet Guild_{:?}", action),
        }
    }
}

async fn update_guild_description(
    player: &PlayerHandle,
    tag: &str,
    description: String,
    conn: &mut Conn,
) {
    if !validate_guild_description(&description) {
        return;
    }

    match conn
        .exec_drop(
            "UPDATE Guild SET `description` = :description WHERE `tag` = :tag",
            params! {
                "description" => description,
                "tag" => tag,
            },
        )
        .await
    {
        Ok(_) => {
            player.send_guild_reply(GuildReply::Updated);
        }
        Err(e) => {
            error!("Error updating guild description: {}", e);
        }
    };
}

async fn update_guild_ranks(player: &PlayerHandle, tag: &str, ranks: [String; 9], conn: &mut Conn) {
    if ranks.iter().any(|rank| !validate_guild_rank(rank)) {
        return;
    }

    let existing_ranks = get_guild_ranks(conn, tag).await;

    for (index, rank) in ranks.iter().enumerate() {
        if existing_ranks[index].eq(rank) {
            continue;
        }

        if let Err(e) = conn
            .exec_drop(
                include_str!("../../../sql/update_guild_rank.sql"),
                params! {
                    "rank" => rank,
                    "tag" => tag,
                    "index" => index,
                },
            )
            .await
        {
            error!("Error updating guild rank: {}", e);
            return;
        }
    }

    player.send_guild_reply(GuildReply::RanksUpdated);
}

async fn get_guild_description(conn: &mut Conn, tag: &str) -> String {
    match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../../../sql/get_guild_description.sql"),
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => {
            let description: String = row.get(0).unwrap();
            if description.is_empty() {
                " ".to_owned()
            } else {
                description
            }
        }
        Err(e) => {
            error!("Error getting guild description: {}", e);
            " ".to_owned()
        }
        _ => " ".to_owned(),
    }
}

async fn get_guild_bank(conn: &mut Conn, tag: &str) -> i32 {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `bank` FROM Guild WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => row.get::<i32, usize>(0).unwrap(),
        Err(e) => {
            error!("Error getting guild bank: {}", e);
            0
        }
        _ => 0,
    }
}

async fn set_guild_bank(conn: &mut Conn, tag: &str, bank: i32) -> Result<(), mysql_async::Error> {
    conn.exec_drop(
        "UPDATE Guild SET `bank` = :bank WHERE `tag` = :tag",
        params! {
            "bank" => bank,
            "tag" => tag,
        },
    )
    .await
}

async fn get_guild_name(conn: &mut Conn, tag: &str) -> Option<String> {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `name` FROM Guild WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => Some(row.get(0).unwrap()),
        Err(e) => {
            error!("Error getting guild name: {}", e);
            None
        }
        _ => None,
    }
}

async fn get_new_member_guild_rank(conn: &mut Conn, tag: &str) -> Option<String> {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `rank` FROM Guild INNER JOIN GuildRank ON GuildRank.`guild_id` = Guild.`id` AND GuildRank.`index` = 8 WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => Some(row.get(0).unwrap()),
        Err(e) => {
            error!("Error getting guild rank: {}", e);
            None
        }
        _ => None,
    }
}

async fn create_guild(
    conn: &mut Conn,
    tag: &str,
    name: &str,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.exec_drop(
        include_str!("../../../sql/create_guild.sql"),
        params! {
            "tag" => tag,
            "name" => name,
            "description" => description,
        },
    )
    .await?;

    let guild_id = conn.last_insert_id().unwrap();

    conn.exec_drop(
        include_str!("../../../sql/create_guild_ranks.sql"),
        params! {
            "guild_id" => guild_id,
            "leader_rank_name" => SETTINGS.guild.default_leader_rank_name.clone(),
            "recruiter_rank_name" => SETTINGS.guild.default_recruiter_rank_name.clone(),
            "new_member_rank_name" => SETTINGS.guild.default_new_member_rank_name.clone(),
        },
    )
    .await?;

    Ok(())
}
