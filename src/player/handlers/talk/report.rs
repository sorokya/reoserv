use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::talk::Report,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn report(buf: PacketBuf, player: PlayerHandle) {
    let mut report = Report::default();
    let reader = StreamReader::new(&buf);
    report.deserialize(&reader);

    debug!("Recv: {:?}", report);
}
