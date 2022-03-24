---
to: src/player/handlers/<%= family %>/<%= action %>.rs
---

use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::<%= family %>::<%= h.capitalize(action) %>, Action, Family},
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn <%= action %>(buf: PacketBuf, player: PlayerHandle) {
    let mut <%= action %> = <%= h.capitalize(action) %>::default();
    let reader = StreamReader::new(&buf);
    <%= action %>.deserialize(&reader);

    debug!("Recv: {:?}", <%= action %>);
}
