macro_rules! send_reply {
    ($player:expr, $reply:expr) => {{
        $player.send(
            eolib::protocol::net::PacketAction::Reply,
            eolib::protocol::net::PacketFamily::Guild,
            &eolib::protocol::net::server::GuildReplyServerPacket {
                reply_code: $reply,
                reply_code_data: None,
            },
        );
    }};
}

mod accept_guild_creation_request;
mod deposit_guild_gold;
mod finish_guild_creation;
mod join_guild;
mod kick_from_guild;
mod leave_guild;
mod open_guild_master;
mod request_to_join_guild;
mod send_guild_create_requests;
mod update_guild_rank;
