use std::sync::Arc;

use eo::{
    data::{EOShort, Serializeable, StreamReader},
    net::{
        packets::{
            client::welcome::Request,
            server::welcome::{Reply, SelectCharacter},
        },
        replies::WelcomeReply,
        Action, Family, ServerSettings,
    },
};

use mysql_async::Conn;
use tokio::sync::Mutex;

use crate::{character::Character, player::Command, world::World, PacketBuf, Tx};

pub async fn request(
    buf: PacketBuf,
    tx: &Tx,
    conn: &mut Conn,
    world: Arc<Mutex<World>>,
    player_id: EOShort,
    characters: Arc<Mutex<Vec<Character>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    // TODO: make sure player is not already logged in

    let mut reply = Reply::new();
    reply.reply = WelcomeReply::SelectCharacter;

    let mut character = Character::load(conn, request.character_id).await?;
    character.player_id = player_id;

    let mut select_character = SelectCharacter::new();
    select_character.player_id = character.player_id;
    select_character.character_id = character.id;
    select_character.map_id = character.map_id;

    let world = world.lock().await;

    {
        let maps = world.maps.lock().unwrap();
        let map_file = maps.get(&character.map_id).unwrap();
        select_character.map_hash = map_file.hash;
        select_character.map_filesize = map_file.size;
    }
    {
        let item_file = world.item_file.lock().unwrap();
        select_character.eif_hash = item_file.hash;
        select_character.eif_length = item_file.len();
    }
    {
        let npc_file = world.npc_file.lock().unwrap();
        select_character.enf_hash = npc_file.hash;
        select_character.enf_length = npc_file.len();
    }
    {
        let spell_file = world.spell_file.lock().unwrap();
        select_character.esf_hash = spell_file.hash;
        select_character.esf_length = spell_file.len();
    }
    {
        let class_file = world.class_file.lock().unwrap();
        select_character.ecf_hash = class_file.hash;
        select_character.ecf_length = class_file.len();
    }

    select_character.name = character.name.to_string();
    select_character.title = match character.title {
        Some(ref title) => title.to_string(),
        None => "".to_string(),
    };
    select_character.guild_name = match character.guild_name {
        Some(ref guild_name) => guild_name.to_string(),
        None => "".to_string(),
    };
    select_character.guild_rank_name = match character.guild_rank_string {
        Some(ref guild_rank_string) => guild_rank_string.to_string(),
        None => "".to_string(),
    };
    select_character.class_id = character.class;
    select_character.guild_tag = match character.guild_tag {
        Some(ref guild_tag) => guild_tag.to_string(),
        None => "".to_string(),
    };
    select_character.admin_level = character.admin_level;
    select_character.level = character.level;
    select_character.experience = character.experience;
    select_character.usage = character.usage;
    select_character.stats = character.get_character_stats_2();
    select_character.paperdoll = character.paperdoll.clone();
    select_character.guild_rank = character.guild_rank_id.unwrap_or(0);
    select_character.settings = ServerSettings {
        jail_map_id: 176,
        unknown_1: 4,
        unknown_2: 24,
        unknown_3: 24,
        light_guide_flood_rate: 10,
        guardian_flood_rate: 10,
        game_master_flood_rate: 10,
        unknown_4: 2,
    };

    select_character.login_message = match character.usage {
        0 => 2,
        _ => 0,
    };

    reply.select_character = Some(select_character);

    characters.lock().await.push(character);

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Welcome,
        reply.serialize(),
    ))?;

    Ok(())
}
