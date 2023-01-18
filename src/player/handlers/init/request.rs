use crate::{
    player::PlayerHandle,
    PacketBuf,
};
use eo::{
    data::{EOByte, Serializeable, StreamReader},
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
pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut packet = client::init::Init::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let sequence_bytes = player.get_sequence_bytes().await;
    let response = stupid_hash(packet.challenge);
    let player_id = player.get_player_id().await;
    let encoding_multiples = player.get_encoding_multiples().await;

    let mut reply = server::init::Init::new();
    reply.reply_code = InitReply::Ok;
    reply.data = InitData::Ok(InitOk {
        response,
        player_id,
        seq_bytes: [sequence_bytes.0 as EOByte, sequence_bytes.1],
        encode_multiple: encoding_multiples[0],
        decode_multiple: encoding_multiples[1],
    });

    debug!("Reply {:?}", reply);

    player.set_state(State::Initialized);
    player.send(PacketAction::Init, PacketFamily::Init, reply.serialize());
}
