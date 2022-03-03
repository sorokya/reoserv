use std::sync::Arc;

use eo::data::EOShort;
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
};

use crate::world::WorldHandle;

use super::{handle_packet::handle_packet, player::Player, Command};

#[derive(Debug)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(player_id: EOShort, socket: TcpStream, world: Arc<Mutex<WorldHandle>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let player = Player::new(player_id, socket, rx, tx.clone(), world);
        tokio::task::Builder::new()
            .name("run_player")
            .spawn(run_player(player));

        Self { tx }
    }
}

async fn run_player(mut player: Player) {
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    trace!("Recv: {:?}", packet);
                    player.queue.get_mut().push_back(packet);
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
                // TODO: really don't like how this reads.. maybe a better way to do this?
                if !player.handle_command(command).await {
                    break;
                }
            }
        }

        if player.busy {
            continue;
        }

        if let Some(packet) = player.queue.get_mut().pop_front() {
            tokio::task::Builder::new()
                .name("handle_packet")
                .spawn(handle_packet(
                    packet,
                    player.id,
                    player.tx.clone(),
                    player.world.clone(),
                ));
        }
    }
}
