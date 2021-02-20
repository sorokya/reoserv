use crate::settings::Settings;
use crate::{client::Client, settings};
use std::net::TcpListener;

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    settings: Settings,
}

impl Server {
    pub fn new() -> std::io::Result<Self> {
        let settings = match Settings::new() {
            Ok(settings) => settings,
            _ => panic!("Failed to load settings!"),
        };

        let address = format!("{}:{}", settings.server.host, settings.server.port);
        let listener = TcpListener::bind(&address)?;
        listener.set_nonblocking(true)?;
        info!("listening at {}", address);
        Ok(Self {
            listener,
            clients: Vec::new(),
            settings,
        })
    }

    pub fn tick(&mut self) -> std::io::Result<()> {
        self.poll()?;

        for client in self.clients.iter_mut() {
            client.tick()?;
        }

        self.drop_dead_clients();

        Ok(())
    }

    fn poll(&mut self) -> std::io::Result<()> {
        match self.listener.accept() {
            Ok((stream, _addr)) => {
                info!("new connection from {}", stream.peer_addr()?,);
                stream.set_nonblocking(true)?;
                self.clients.push(Client::new(stream));
            }
            _ => {}
        }

        Ok(())
    }

    fn drop_dead_clients(&mut self) {
        for i in self.clients.len()..0 {
            if self.clients[i].closed {
                self.clients.remove(i);
            }
        }
    }
}
