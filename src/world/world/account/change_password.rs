use eo::{
    data::{EOInt, i32, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};
use mysql_async::{prelude::*, Params, Row};

use super::{
    super::World,
    password_hash::{generate_password_hash, validate_password},
};

use super::account_exists::account_exists;

impl World {
    pub fn change_password(
        &self,
        player_id: i32,
        username: String,
        current_password: String,
        new_password: String,
    ) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.to_owned(),
            None => return,
        };

        let conn = self.pool.get_conn();
        tokio::spawn(async move {
            let mut conn = match conn.await {
                Ok(conn) => conn,
                Err(e) => {
                    player.close(format!("Error getting connection from pool: {}", e));
                    return;
                }
            };

            let exists = match account_exists(&mut conn, &username).await {
                Ok(exists) => exists,
                Err(e) => {
                    player.close(format!("Error checking if account exists: {}", e));
                    return;
                }
            };

            if !exists {
                let mut builder = StreamBuilder::new();
                builder.add_short(5);
                builder.add_string("NO");
                player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
                return;
            }

            let row = match conn
                .exec_first::<Row, &str, Params>(
                    include_str!("../../../sql/get_password_hash.sql"),
                    params! {
                        "name" => &username,
                    },
                )
                .await
            {
                Ok(row) => row,
                Err(e) => {
                    error!("Error getting password hash: {}", e);
                    let mut builder = StreamBuilder::new();
                    builder.add_short(5);
                    builder.add_string("NO");
                    player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
                    return;
                }
            }
            .unwrap();

            let password_hash: String = row.get("password_hash").unwrap();
            if !validate_password(&username, &current_password, &password_hash) {
                let mut builder = StreamBuilder::new();
                builder.add_short(5);
                builder.add_string("NO");
                player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
                return;
            }

            let account_id: EOInt = row.get("id").unwrap();

            let password_hash = generate_password_hash(&username, &new_password);
            if let Err(e) = conn
                .exec_drop(
                    include_str!("../../../sql/update_password_hash.sql"),
                    params! {
                        "id" => account_id,
                        "password_hash" => &password_hash,
                    },
                )
                .await
            {
                player.close(format!("Error updating password hash: {}", e));
                return;
            }

            let mut builder = StreamBuilder::new();
            builder.add_short(6);
            builder.add_string("NO");
            player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
        });
    }
}
