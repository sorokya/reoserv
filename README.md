![Rust](https://github.com/sorokya/reoserv/workflows/Rust/badge.svg)

# REOSERV

The rust powered [Endless Online](https://game.eoserv.net/) server emulator!


# Database setup

0. You should install docker-compose
1. Create a .env file with the following (replacing <PASSWORD>s with your own secure password)

```
MYSQL_ROOT_PASSWORD=<ROOT_PASSWORD>
MYSQL_REOSERV_PASSWORD=<REOSERV_PASSWORD>
```

2. Start the container

`docker-compose up`

# Start server

You can run the server with

`cargo run`

(if you want more logging then set the `RUST_LOG` environment variable to either `trace`, or `debug`)
