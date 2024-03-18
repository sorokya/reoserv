macro_rules! send_reply {
    ($player:expr, $reply:expr) => {{
        let mut writer = eolib::data::EoWriter::new();
        let packet = eolib::protocol::net::server::GuildReplyServerPacket {
            reply_code: $reply,
            reply_code_data: None,
        };

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildReplyServerPacket: {}", e);
            return;
        }

        let _ = $player
            .bus
            .send(
                eolib::protocol::net::PacketAction::Reply,
                eolib::protocol::net::PacketFamily::Guild,
                writer.to_byte_array(),
            )
            .await;
    }};
}

mod accept_guild_join_request;
mod add_guild_creation_player;
mod assign_guild_rank;
mod create_guild;
mod guild_exists;
pub use guild_exists::guild_exists;
mod kick_guild_member;
mod leave_guild;
mod request_guild_creation;
mod request_guild_details;
mod request_guild_info;
mod request_guild_memberlist;
mod update_guild;
mod validate_guild_tag;
pub use validate_guild_tag::validate_guild_tag;
mod validate_guild_name;
pub use validate_guild_name::validate_guild_name;
mod validate_guild_description;
pub use validate_guild_description::validate_guild_description;
mod validate_guild_rank;
pub use validate_guild_rank::validate_guild_rank;
