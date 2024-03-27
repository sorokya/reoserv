use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{MessageOpenServerPacket, MusicPlayerServerPacket},
            PacketAction, PacketFamily,
        },
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
                let map_id = match args.first() {
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
                let item_id = match args.first() {
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
                let item_id = match args.first() {
                    Some(Arg::Int(item_id)) => *item_id,
                    _ => return,
                };

                let amount = match args.get(1) {
                    Some(Arg::Int(amount)) => *amount,
                    _ => 1,
                };

                map.lose_item(self.id, item_id, amount);
            }
            "SetClass" => {}
            "PlayMusic" => {
                if let Some(Arg::Int(sound_id)) = args.first() {
                    let packet = MusicPlayerServerPacket {
                        sound_id: *sound_id,
                    };

                    let mut writer = EoWriter::new();

                    if let Err(e) = packet.serialize(&mut writer) {
                        error!("Error serializing MusicPlayerServerPacket: {}", e);
                        return;
                    }

                    let _ = self
                        .bus
                        .send(
                            PacketAction::Player,
                            PacketFamily::Music,
                            writer.to_byte_array(),
                        )
                        .await;
                }
            }
            "PlaySound" => {
                if let Some(Arg::Int(sound_id)) = args.first() {
                    let packet = MusicPlayerServerPacket {
                        sound_id: *sound_id,
                    };

                    let mut writer = EoWriter::new();

                    if let Err(e) = packet.serialize(&mut writer) {
                        error!("Error serializing MusicPlayerServerPacket: {}", e);
                        return;
                    }

                    let _ = self
                        .bus
                        .send(
                            PacketAction::Player,
                            PacketFamily::Music,
                            writer.to_byte_array(),
                        )
                        .await;
                }
            }
            "ShowHint" => {
                let message = match args.first() {
                    Some(Arg::Str(message)) => message,
                    _ => return,
                };

                let packet = MessageOpenServerPacket {
                    message: message.to_owned(),
                };

                let mut writer = EoWriter::with_capacity(message.len());

                if packet.serialize(&mut writer).is_err() {
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
            "GiveExp" => {
                if let Some(Arg::Int(amount)) = args.first() {
                    map.award_experience(self.id, *amount);
                }
            }
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
