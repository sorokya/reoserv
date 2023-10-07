use crate::player::{ClientState, PlayerHandle};
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
pub async fn request(
    reader: StreamReader,
    player: PlayerHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = client::init::Init::default();
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let sequence_bytes = player.get_sequence_bytes().await?;
    let response = stupid_hash(packet.challenge);
    let player_id = player.get_player_id().await?;
    let encoding_multiples = player.gen_encoding_multiples().await?;

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

    player.set_state(ClientState::Initialized);

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());

    Ok(())
}
