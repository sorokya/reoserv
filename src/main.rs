extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod character;
mod map;
mod world;
use tokio::net::TcpListener;
use world::World;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    println!("__________
\\______   \\ ____  ____  ______ ______________  __
 |       _// __ \\/  _ \\/  ___// __ \\_  __ \\  \\/ /
 |    |   \\  ___(  <_> )___ \\\\  ___/|  | \\/\\   /
 |____|_  /\\___  >____/____  >\\___  >__|    \\_/
        \\/     \\/          \\/     \\/\nThe rusty endless online server: v0.0.0\n");
    let mut world = World::new();
    world.load_maps(282).await?;
    world.load_pub_files().await?;

    let listener = TcpListener::bind("0.0.0.0:8078").await?;
    info!("listening at 0.0.0.0:8078");

    loop {
        let (socket, addr) = listener.accept().await?;

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            info!("connection accepted ({})", addr);
            // if let Err(e) = process(state, stream, addr).await {
            // }
        });
    }
}


