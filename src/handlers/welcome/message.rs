

use eo::{
    data::{EOChar, EOShort, Serializeable, StreamReader},
    net::{
        packets::{
            client::welcome::Message,
            server::welcome::{EnterGame, Reply},
        },
        replies::WelcomeReply,
        Action, CharacterMapInfo, Family, NearbyInfo, PaperdollB000A0HSW, Weight,
    },
};

use crate::{character::Character, player::Command, PacketBuf, Tx};

pub async fn message(
    buf: PacketBuf,
    tx: &Tx,
    character: &Character,
    player_id: EOShort,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Message::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    let mut reply = Reply::new();
    reply.reply = WelcomeReply::EnterGame;

    let mut enter_game = EnterGame::new();
    enter_game.news[0] = "Welcome to my server!".to_string();
    enter_game.news[1] = "Powered by rust!".to_string();
    enter_game.weight = Weight {
        current: 0,
        max: 70,
    };
    enter_game.items = vec![];
    enter_game.spells = vec![];
    enter_game.nearby_info = NearbyInfo::default();
    enter_game.nearby_info.characters.push(CharacterMapInfo {
        name: character.name.to_string(),
        id: player_id,
        map_id: character.map_id,
        coords: character.coords,
        direction: character.direction,
        class_id: character.class,
        guild_tag: String::default(),
        level: character.level,
        gender: character.gender,
        hair_style: character.hair_style as EOChar,
        hair_color: character.hair_color as EOChar,
        race: character.race,
        max_hp: character.max_hp,
        hp: character.hp,
        max_tp: character.max_tp,
        tp: character.tp,
        paperdoll: PaperdollB000A0HSW {
            boots: character.paperdoll.boots,
            armor: character.paperdoll.armor,
            hat: character.paperdoll.hat,
            shield: character.paperdoll.shield,
            weapon: character.paperdoll.weapon,
        },
        sit_state: character.sit_state,
        invisible: character.hidden,
    });
    reply.enter_game = Some(enter_game);

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Welcome,
        reply.serialize(),
    ))?;

    Ok(())
}
