use eo::{
    data::{Serializeable, StreamReader, StreamBuilder},
    protocol::{
        client::character::Request,
        server::character::{Reply, ReplyData, ReplyFull3},
        CharacterReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn request(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
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

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);

    player.send(
        PacketAction::Reply,
        PacketFamily::Character,
        builder.get(),
    );
}
