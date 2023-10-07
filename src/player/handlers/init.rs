use eo::{
    data::{EOByte, Serializeable, StreamBuilder, StreamReader},
    net::stupid_hash,
    protocol::{
        client,
        server::{
            self,
            init::{InitData, InitOk},
        },
        InitReply, PacketAction, PacketFamily,
    },
};

use crate::player::{ClientState, PlayerHandle};

async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut packet = client::init::Init::default();
    packet.deserialize(&reader);

    let sequence_bytes = match player.get_sequence_bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            player.close(format!("Failed to get sequence bytes: {}", e));
            return;
        }
    };

    let response = stupid_hash(packet.challenge);
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            player.close(format!("Failed to get player id: {}", e));
            return;
        }
    };

    let encoding_multiples = match player.gen_encoding_multiples().await {
        Ok(multiples) => multiples,
        Err(e) => {
            player.close(format!("Failed to generate encoding multiples: {}", e));
            return;
        }
    };

    let mut reply = server::init::Init::new();
    reply.reply_code = InitReply::Ok;
    reply.data = InitData::Ok(InitOk {
        response,
        player_id,
        seq_bytes: [sequence_bytes.0 as EOByte, sequence_bytes.1],
        encode_multiple: encoding_multiples[0],
        decode_multiple: encoding_multiples[1],
    });

    player.set_state(ClientState::Initialized);

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());
}

pub async fn init(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Init => request(reader, player).await,
        _ => error!("Unhandled packet Init_{:?}", action),
    }
}
