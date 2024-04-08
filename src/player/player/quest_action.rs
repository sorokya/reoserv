use eolib::{
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
            "SetClass" => {
                let class_id = match args.first() {
                    Some(Arg::Int(class_id)) => *class_id,
                    _ => return,
                };

                map.set_class(self.id, class_id);
            }
            "PlayMusic" => {
                if let Some(Arg::Int(sound_id)) = args.first() {
                    let _ = self
                        .bus
                        .send(
                            PacketAction::Player,
                            PacketFamily::Music,
                            MusicPlayerServerPacket {
                                sound_id: *sound_id,
                            },
                        )
                        .await;
                }
            }
            "PlaySound" => {
                if let Some(Arg::Int(sound_id)) = args.first() {
                    let _ = self
                        .bus
                        .send(
                            PacketAction::Player,
                            PacketFamily::Music,
                            MusicPlayerServerPacket {
                                sound_id: *sound_id,
                            },
                        )
                        .await;
                }
            }
            "ShowHint" => {
                let message = match args.first() {
                    Some(Arg::Str(message)) => message,
                    _ => return,
                };

                let _ = self
                    .bus
                    .send(
                        PacketAction::Open,
                        PacketFamily::Message,
                        MessageOpenServerPacket {
                            message: message.to_owned(),
                        },
                    )
                    .await;
            }
            "GiveExp" => {
                if let Some(Arg::Int(amount)) = args.first() {
                    map.award_experience(self.id, *amount);
                }
            }
            "GiveKarma" => {
                if let Some(Arg::Int(amount)) = args.first() {
                    map.give_karma(self.id, *amount);
                }
            }
            "RemoveKarma" => {
                if let Some(Arg::Int(amount)) = args.first() {
                    map.remove_karma(self.id, *amount);
                }
            }
            "Quake" => {
                if let Some(Arg::Int(magnitude)) = args.first() {
                    if *magnitude == 8 {
                        self.world.quake(*magnitude);
                    } else {
                        map.quake(*magnitude);
                    }
                }
            }
            "EffectOnPlayer" => {
                let effect_id = match args.first() {
                    Some(Arg::Int(effect_id)) => *effect_id,
                    _ => return,
                };

                map.effect_on_player(self.id, effect_id);
            }
            "EffectOnCoord" => {
                let effect_id = match args.first() {
                    Some(Arg::Int(effect_id)) => *effect_id,
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

                map.effect_on_coord(Coords { x, y }, effect_id);
            }
            _ => {}
        }
    }
}
