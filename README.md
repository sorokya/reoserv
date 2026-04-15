[![Rust](https://github.com/sorokya/reoserv/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/sorokya/reoserv/actions/workflows/rust.yml)

<img src="https://raw.githubusercontent.com/sorokya/reoserv/master/assets/logo.webp" alt="REOSERV" style="width:500px"/>

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
> Set up and configure your database before starting the server. You can use either MySQL/MariaDB or SQLite.
> Edit `config/Config.toml` to match your production database before creating a release build.

## Database setup and configuration

Reoserv supports both MySQL/MariaDB and SQLite.

1. Choose a database driver in `config/Config.toml` (or `config/Config.local.toml`):
    - `driver = "mysql"` for MySQL/MariaDB
    - `driver = "sqlite"` for SQLite

2. Configure database connection settings:
    - For MySQL/MariaDB, set `host`, `port`, `name`, `username`, and `password`.
    - For SQLite, set `name` (the server will use `<name>.db` in the working directory).

3. Start the server normally. It will create the migration log table and apply any missing migrations from `data/migrations/` automatically:
    ```sh
    cargo run
    ```

> [!WARNING]
> Back up your database first before upgrading an older installation. Startup now applies pending migrations automatically, including legacy schema transitions.

4. If you choose MySQL/MariaDB and are using the provided Compose setup, start only the database service with:
    ```sh
    docker compose up -d db
    ```

## Docker Compose (Reoserv + MariaDB)

The provided `compose.yml` starts both services:

- `db`: MariaDB database
- `reoserv`: Reoserv server container

Before starting, make sure your `config/Config.toml` (or mounted `config/Config.local.toml`) uses:

- `driver = "mysql"`
- `host = "db"`
- `port = "3306"`
- Matching `name`, `username`, and `password` values

Build/start the stack:

```sh
docker compose up -d --build
```

Start the server in the container to let startup migrations create or upgrade the schema:

```sh
docker compose run --rm reoserv ./reoserv
```

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
