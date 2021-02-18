use crate::client::Client;
use std::net::TcpListener;

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
}

impl Server {
    pub fn new() -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:8078")?;
        listener.set_nonblocking(true)?;
        info!("listening at 127.0.0.1:8078");
        Ok(Self {
            listener,
            clients: Vec::new(),
        })
    }

    pub fn tick(&mut self) -> std::io::Result<()> {
        self.poll()?;
        for client in self.clients.iter_mut() {
            client.tick()?;
        }
        Ok(())
    }

    fn poll(&mut self) -> std::io::Result<()> {
        match self.listener.accept() {
            Ok((stream, _addr)) => {
                info!("new connection from {}", stream.peer_addr()?);
                self.clients.push(Client::new(stream));
            }
            _ => {}
        }

        Ok(())
    }
}
