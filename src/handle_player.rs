use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use eo::{
    data::{EOShort, StreamBuilder},
    net::{Action, Family},
};
use mysql_async::Pool;
use tokio::{net::TcpStream, sync::Mutex};

use crate::{
    character::Character,
    handle_packet::handle_packet,
    player::{Command, Player, State},
    world::World,
    PacketBuf, Players, Tx,
};

pub async fn handle_player(
    world: Arc<Mutex<World>>,
    players: Players,
    characters: Arc<Mutex<Vec<Character>>>,
    active_account_ids: Arc<Mutex<Vec<u32>>>,
    socket: TcpStream,
    db_pool: Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    let player_id = {
        let players = players.lock().await;
        get_next_player_id(&players, 1)
    };

    let ip = socket.peer_addr()?.ip().to_string();
    let mut player = Player::new(players.clone(), socket, player_id, ip).await;
    let mut queue: RefCell<VecDeque<PacketBuf>> = RefCell::new(VecDeque::new());

    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    trace!("Recv: {:?}", packet);
                    queue.get_mut().push_back(packet);
                },
                Some(Err(e)) => {
                    match e.kind() {
                        std::io::ErrorKind::BrokenPipe => {
                            info!("player {} connection closed by peer", player.id);
                            break;
                        },
                        _ => {
                            info!("player {} connection closed due to unknown error: {:?}", player.id, e);
                            break;
                        }
                    }
                },
                None => {
                }
            },
            Some(command) = player.rx.recv() => {
                match command {
                    Command::Send(action, family, data) => {
                        player.bus.send(action, family, data).await?;
                    },
                    Command::Close(reason) => {
                        info!("player {} connection closed: {}", player.id, reason);
                        break;
                    }
                    Command::SetState(state) => {
                        player.state = state;
                        match player.state {
                            State::LoggedIn(account_id, num_of_characters) => {
                                player.account_id = account_id;
                                player.num_of_characters = num_of_characters;
                                active_account_ids.lock().await.push(account_id);
                            },
                            State::Playing(character_id) => {
                                player.character_id = character_id;
                            },
                            _ => {}
                        }
                    }
                    Command::NewCharacter => {
                        player.num_of_characters += 1;
                    }
                    Command::DeleteCharacter => {
                        player.num_of_characters -= 1;
                    }
                    Command::Ping => {
                        if player.bus.need_pong {
                            info!("player {} connection closed: ping timeout", player.id);
                            break;
                        } else {
                            player.bus.sequencer.ping_new_sequence();
                            let sequence = player.bus.sequencer.get_update_sequence_bytes();
                            let mut builder = StreamBuilder::with_capacity(3);
                            builder.add_short(sequence.0);
                            builder.add_char(sequence.1);
                            player.bus.need_pong = true;
                            player.bus.send(Action::Player, Family::Connection, builder.get()).await?;
                        }
                    },
                    Command::Pong => {
                        player.bus.need_pong = false;
                    },
                    _ => {
                        error!("unhandled command: {:?}", command);
                    }
                }
            },
        }

        if let Some(packet) = queue.get_mut().pop_front() {
            let db_pool = db_pool.clone();
            match handle_packet(
                packet,
                &mut player,
                world.clone(),
                players.clone(),
                active_account_ids.clone(),
                db_pool,
                characters.clone(),
            )
            .await
            {
                Ok(()) => {}
                Err(e) => {
                    error!("error handling packet: {:?}", e);
                }
            }
        }
    }

    players.lock().await.remove(&player.id);

    if player.character_id != 0 {
        let mut characters = characters.lock().await;
        characters.retain(|character| character.player_id != player.id);
    }

    if player.account_id != 0 {
        let mut accounts = active_account_ids.lock().await;
        accounts.retain(|&x| x != player.account_id);
    }

    Ok(())
}

fn get_next_player_id(
    players: &tokio::sync::MutexGuard<HashMap<EOShort, Tx>>,
    seed: EOShort,
) -> EOShort {
    let id = seed;
    for player_id in players.iter().map(|c| *c.0) {
        if player_id == id {
            return get_next_player_id(players, id + 1);
        }
    }
    id
}
