[![Rust](https://github.com/sorokya/reoserv/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/sorokya/reoserv/actions/workflows/rust.yml)

# REOSERV

The rust powered [Endless Online](https://game.eoserv.net/) server emulator!

# Building

The only depency is rust and cargo. I recommend using [rustup](https://rustup.rs/) to install both.

To build the server simply run

`cargo build`

or for release build

`cargo build --release`

# Database setup

0. You should install docker-compose
1. Create a .env file with the following (replacing <PASSWORD>s with your own secure password)

```
MYSQL_ROOT_PASSWORD=<ROOT_PASSWORD>
MYSQL_REOSERV_PASSWORD=<REOSERV_PASSWORD>
```

2. Start the container

`docker-compose up`

# Configure the server

_You can either edit Config.toml directly or edit a copy Config.local.toml_

The only required change right now is the settings for the database connection.

If you're using the provided docker-compose file then all you need to do is set
password to the same password you set to for `MYSQL_REOSERV_PASSWORD` in the .env file.

# Start server

You can run the server with

`cargo run`

(if you want more logging then set the `RUST_LOG` environment variable to one
of the following:

- debug: The human readable data structures for every packet will be printed
- trace: The raw byte arrays for every packet will be printed
