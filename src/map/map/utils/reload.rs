use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::{Emf, MapTileSpec},
        net::{
            server::{
                InitInitServerPacket, InitInitServerPacketReplyCodeData,
                InitInitServerPacketReplyCodeDataMapMutation, InitReply, MapFile,
            },
            PacketAction, PacketFamily,
        },
        Coords,
    },
};

use crate::map::Door;

use super::{super::Map, create_chests};

impl Map {
    pub fn reload(&mut self, file: Box<Emf>, file_size: i32) {
        self.npcs_initialized = false;
        self.npcs.clear();

        self.has_timed_spikes = file.tile_spec_rows.iter().any(|row| {
            row.tiles
                .iter()
                .any(|tile| tile.tile_spec == MapTileSpec::TimedSpikes)
        });

        self.has_jukebox = file.tile_spec_rows.iter().any(|row| {
            row.tiles
                .iter()
                .any(|tile| tile.tile_spec == MapTileSpec::Jukebox)
        });

        self.doors.clear();

        for row in &file.warp_rows {
            for tile in &row.tiles {
                if tile.warp.door > 0 {
                    self.doors.push(Door::new(
                        Coords {
                            x: tile.x,
                            y: row.y,
                        },
                        tile.warp.door,
                    ));
                }
            }
        }

        self.chests = create_chests(self.id, &file);
        self.arena_ticks = 0;
        self.arena_players.clear();
        self.quake_ticks = 0;
        self.quake_strength = None;
        self.quake_rate = None;
        self.jukebox_player = None;
        self.jukebox_ticks = 0;
        self.wedding = None;
        self.wedding_ticks = 0;

        let mut writer = EoWriter::new();

        let file = *file;

        if let Err(e) = file.serialize(&mut writer) {
            error!("Failed to serialize Emf: {}", e);
            return;
        }

        let packet = InitInitServerPacket {
            reply_code: InitReply::MapMutation,
            reply_code_data: Some(InitInitServerPacketReplyCodeData::MapMutation(
                InitInitServerPacketReplyCodeDataMapMutation {
                    map_file: MapFile {
                        content: writer.to_byte_array().to_vec(),
                    },
                },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize InitInitServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        self.file = file;
        self.file_size = file_size;

        for character in self.characters.values() {
            let player = match character.player {
                Some(ref player) => player,
                None => continue,
            };

            player.send_buf(PacketAction::Init, PacketFamily::Init, buf.clone());
        }
    }
}
