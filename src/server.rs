use std::net::TcpListener;

use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use eo::data::EOShort;

use crate::client::Client;
use crate::settings::Settings;

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    settings: Settings,
    db: MysqlConnection,
}

impl Server {
    pub fn new() -> std::io::Result<Self> {
        let settings = match Settings::new() {
            Ok(settings) => settings,
            _ => panic!("Failed to load settings!"),
        };

        let db_url = format!(
            "mysql://{}:{}@{}/{}",
            settings.database.username,
            settings.database.password,
            settings.database.host,
            settings.database.name
        );

        let db =
            MysqlConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url));

        let address = format!("{}:{}", settings.server.host, settings.server.port);
        let listener = TcpListener::bind(&address)?;
        listener.set_nonblocking(true)?;
        info!("listening at {}", address);
        Ok(Self {
            listener,
            clients: Vec::new(),
            settings,
            db,
        })
    }

    pub fn tick(&mut self) -> std::io::Result<()> {
        self.poll()?;

        for client in self.clients.iter_mut() {
            client.tick(&self.db, &self.settings)?;
        }

        self.drop_dead_clients();

        Ok(())
    }

    fn poll(&mut self) -> std::io::Result<()> {
        match self.listener.accept() {
            Ok((stream, _addr)) => {
                if self.clients.len() as u32 >= self.settings.server.max_connections {
                    info!("connection refused: server full");
                } else {
                    info!(
                        "new connection from {} ({}/{})",
                        stream.peer_addr()?,
                        self.clients.len() + 1,
                        self.settings.server.max_connections
                    );
                    stream.set_nonblocking(true)?;
                    self.clients
                        .push(Client::new(stream, self.get_next_player_id(1)));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn get_next_player_id(&self, seed: EOShort) -> EOShort {
        let id = seed;
        for player_id in self.clients.iter().map(|c| c.player_id) {
            if player_id == id {
                return self.get_next_player_id(id + 1);
            }
        }
        id
    }

    fn drop_dead_clients(&mut self) {
        for i in self.clients.len()..0 {
            if self.clients[i].closed {
                self.clients.remove(i);
            }
        }
    }
}
