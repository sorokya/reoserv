use crate::{
    player::{PlayerHandle, State},
    PacketBuf,
};
use eo::{
    data::{EOByte, Serializeable, StreamReader},
    net::{
        packets::{
            client::init::Request,
            server::init::{Reply, ReplyOk},
        },
        replies::InitReply,
        stupid_hash, Action, Family,
    },
};
pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut packet = Request::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let mut reply = Reply::new();
    reply.reply_code = InitReply::OK;

    let mut init_ok = ReplyOk::new();
    init_ok.challenge_response = stupid_hash(packet.challenge);

    let player_id = player.get_player_id().await;
    init_ok.player_id = player_id;

    let sequence_bytes = player.get_sequence_bytes().await;
    init_ok.sequence_bytes = [sequence_bytes.0 as EOByte, sequence_bytes.1];
    init_ok.encoding_multiples = player.gen_encoding_multiples().await;

    debug!("Reply code: {:?}, data: {:?}", reply.reply_code, init_ok);

    reply.reply = Box::new(init_ok);

    player.set_state(State::Initialized);
    player.send(Action::Init, Family::Init, reply.serialize());
}
