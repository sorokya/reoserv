use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use eo::{
    data::{EOShort, StreamBuilder},
    net::{Action, Family},
};
use mysql_async::Pool;
use tokio::net::TcpStream;

use crate::{
    handle_packet::handle_packet,
    player::{Command, Player},
    PacketBuf, Players, Tx,
};

pub async fn handle_player(
    players: Players,
    socket: TcpStream,
    db_pool: Pool,
) -> Result<(), Box<dyn std::error::Error>> {
    let player_id = {
        let players = players.lock().await;
        get_next_player_id(&players, 1)
    };

    let player_ip = socket.peer_addr()?.ip().to_string();
    let mut player = Player::new(players.clone(), socket, player_id).await;
    let mut queue: RefCell<VecDeque<PacketBuf>> = RefCell::new(VecDeque::new());
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    debug!("Recv: {:?}", packet);
                    queue.get_mut().push_back(packet);
                },
                Some(Err(e)) => {
                    error!("error receiving packet: {:?}", e);
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
                        info!("player {} connection closed: {}", player_id, reason);
                        break;
                    }
                    Command::Ping => {
                        if player.bus.need_pong {
                            info!("player {} connection closed: ping timeout", player_id);
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
            match handle_packet(player_id, packet, &mut player.bus, players.clone(), db_pool, &player_ip).await
            {
                Ok(()) => {}
                Err(e) => {
                    error!("error handling packet: {:?}", e);
                }
            }
        }
    }

    players.lock().await.remove(&player_id);

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
