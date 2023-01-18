use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::character::Request,
        server::character::{Reply, ReplyData, ReplyFull3},
        CharacterReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    let reply = if request.new != "NEW" {
        Reply {
            reply_code: CharacterReply::Full3,
            data: ReplyData::Full3(ReplyFull3 {
                no: "NO".to_string(),
            }),
        }
    } else {
        match world.request_character_creation(player.clone()).await {
            Ok(reply) => reply,
            Err(_) => Reply {
                reply_code: CharacterReply::Full3,
                data: ReplyData::Full3(ReplyFull3 {
                    no: "NO".to_string(),
                }),
            },
        }
    };

    debug!("Reply: {:?}", reply);

    player.send(
        PacketAction::Reply,
        PacketFamily::Character,
        reply.serialize(),
    );
}
