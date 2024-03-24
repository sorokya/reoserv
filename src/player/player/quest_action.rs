use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::MessageOpenServerPacket, PacketAction, PacketFamily},
        Coords,
    },
};
use eoplus::Arg;

use super::Player;

impl Player {
    pub async fn quest_action(&mut self, action: String, args: Vec<Arg>) {
        let map = match self.map {
            Some(ref map) => map,
            None => return,
        };

        match action.as_str() {
            "SetMap" => {
                let map_id = match args.get(0) {
                    Some(Arg::Int(map_id)) => *map_id,
                    _ => return,
                };

                let x = match args.get(1) {
                    Some(Arg::Int(x)) => *x,
                    _ => return,
                };

                let y = match args.get(2) {
                    Some(Arg::Int(y)) => *y,
                    _ => return,
                };

                self.request_warp(map_id, Coords { x, y }, false, None)
                    .await;
            }
            "GiveItem" => {
                let item_id = match args.get(0) {
                    Some(Arg::Int(item_id)) => *item_id,
                    _ => return,
                };

                let amount = match args.get(1) {
                    Some(Arg::Int(amount)) => *amount,
                    _ => 1,
                };

                map.give_item(self.id, item_id, amount);
            }
            "RemoveItem" => {
                let item_id = match args.get(0) {
                    Some(Arg::Int(item_id)) => *item_id,
                    _ => return,
                };

                let amount = match args.get(1) {
                    Some(Arg::Int(amount)) => *amount,
                    _ => 1,
                };

                map.lose_item(self.id, item_id, amount);
            }
            "End" => {}
            "Reset" => {}
            "SetClass" => {}
            "PlayMusic" => {}
            "PlaySound" => {}
            "ShowHint" => {
                let message = match args.get(0) {
                    Some(Arg::Str(message)) => message,
                    _ => return,
                };

                let packet = MessageOpenServerPacket {
                    message: message.to_owned(),
                };

                let mut writer = EoWriter::with_capacity(message.len());

                if let Err(_) = packet.serialize(&mut writer) {
                    return;
                }

                let _ = self
                    .bus
                    .send(
                        PacketAction::Open,
                        PacketFamily::Message,
                        writer.to_byte_array(),
                    )
                    .await;
            }
            "GiveExp" => {}
            "RemoveExp" => {}
            "GiveKarma" => {}
            "RemoveKarma" => {}
            "Quake" => {}
            "EffectOnPlayer" => {}
            "EffectOnCoord" => {}
            _ => {}
        }
    }
}
