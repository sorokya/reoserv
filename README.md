[![Rust](https://github.com/sorokya/reoserv/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/sorokya/reoserv/actions/workflows/rust.yml)

# REOSERV

The Rust-powered server emulator for [Endless Online](https://endless-online.com) 🦀

## Prerequisites

Before building the server, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/)
- [Cargo](https://doc.rust-lang.org/cargo/appendix/glossary.html#cargo)

I recommend using [rustup](https://rustup.rs/) to install Rust and Cargo.

## Building

To build the server, simply run:

```sh
cargo build
```

For a release build, use:

```sh
cargo build --release
```

> [!NOTE]
> Set up and configure a MySQL database before starting the server. See the section below for instructions.
> Edit `config/Config.toml` to match your production database before creating a release build.

## Database setup and configuration

We use a MySQL database to store game data.

1. If you don't have a MySQL database set up, you can run this Docker command to create one:
    ```sh
    docker run --name reoserv-db \
        -e MYSQL_ROOT_PASSWORD="CHANGEME" \
        -e MYSQL_PASSWORD="CHANGEME" \
        -e MYSQL_USER="reoserv" \
        -e MYSQL_DATABASE="reoserv" \
        -e TZ="UTC" \
        -p "3306:3306" \
        -v ./db-init/:/docker-entrypoint-initdb.d/ \
        --restart unless-stopped \
        -d mariadb:latest
    ```
    Replace `CHANGEME` with your own secure passwords.

2. Edit the database connection settings in `config/Config.toml` or in a copy of it (`config/Config.local.toml`) accordingly before building / running the server.

## Start the server

To run the server, use:

```sh
cargo run
```

## Setup quests, NPCs and items for your server

See [our documentation](https://reoserv.net/docs) for instructions on how to setup quests, NPCs, items and more for your server.

- [Classes, Items, Spells, NPCs](https://reoserv.net/docs/pubs)
- [Maps](https://reoserv.net/docs/maps)
- [Quests](https://reoserv.net/docs/quests)

## Setup the Endless Online client

See `eo-client/README.md` for instructions
