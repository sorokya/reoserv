//! Automated in-game test bots for reoserv.
//!
//! Each bot connects to a running reoserv instance, completes the full EO
//! client handshake (init → connection accept → account creation → login →
//! character creation → character select → enter game), and then idles in-game
//! while logging traffic. Useful for load testing and automated gameplay checks.
//!
//! Wire protocol (EO / v28):
//!   [2-byte length][action][family][sequence][data...]
//!   - The length covers everything after itself and is *not* encrypted.
//!   - The `sequence` byte is only present for non-Init families.
//!   - After init, the whole payload (action..data) is encrypted with the
//!     `client_encryption_multiple` (outgoing) / `server_encryption_multiple`
//!     (incoming) negotiated during the init handshake.
//!
//! Usage:
//!   cargo run --bin reoserv-bots -- --bots 5 --host 127.0.0.1:8078 --account-prefix bot
//!
//! Note: to run more than a couple bots from the same IP, disable the server's
//! per-IP rate limits in `config/Config.toml` (`max_connections_per_ip = 0`,
//! `ip_reconnect_limit = 0`), otherwise extra bots are refused with
//! "reconnected too quickly".

use std::time::Duration;

use anyhow::{Result, anyhow};
use bytes::{BufMut, Bytes, BytesMut};
use eolib::{
    data::{EoReader, EoSerialize, EoWriter, decode_number, encode_number},
    encrypt::{decrypt_packet, encrypt_packet, server_verification_hash},
    packet::{Sequencer, get_init_sequence_start},
    protocol::{
        Gender,
        net::{
            PacketAction, PacketFamily, Version,
            client::{
                AccountCreateClientPacket, AccountRequestClientPacket, CharacterCreateClientPacket,
                CharacterRequestClientPacket, ConnectionAcceptClientPacket, InitInitClientPacket,
                LoginRequestClientPacket, WelcomeMsgClientPacket, WelcomeRequestClientPacket,
            },
            server::{
                AccountReply, AccountReplyServerPacket, CharacterReply, CharacterReplyServerPacket,
                CharacterReplyServerPacketReplyCodeData, InitInitServerPacket,
                InitInitServerPacketReplyCodeData, InitReply, LoginReply, LoginReplyServerPacket,
                LoginReplyServerPacketReplyCodeData, WelcomeReplyServerPacket,
                WelcomeReplyServerPacketWelcomeCodeData,
            },
        },
    },
};
use rand::RngExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// A non-"deep" client version so the server skips email-validation and config packets.
const VERSION: Version = Version {
    major: 0,
    minor: 0,
    patch: 28,
};

struct Bot {
    stream: TcpStream,
    sequencer: Sequencer,
    client_encryption_multiple: u8,
    server_encryption_multiple: u8,
    player_id: i32,
}

impl Bot {
    async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        stream.set_nodelay(true).ok();
        Ok(Self {
            stream,
            sequencer: Sequencer::new(0),
            client_encryption_multiple: 0,
            server_encryption_multiple: 0,
            player_id: 0,
        })
    }

    async fn send_packet(
        &mut self,
        action: PacketAction,
        family: PacketFamily,
        data: &[u8],
    ) -> Result<()> {
        let mut payload = BytesMut::new();
        payload.put_u8(u8::from(action));
        payload.put_u8(u8::from(family));

        // The sequencer advances on every send — including the Init handshake —
        // matching the reference client. The sequence byte itself is only written
        // for non-Init packets (Init carries no sequence byte).
        let seq = self.sequencer.next_sequence();
        if family != PacketFamily::Init {
            // The sequence is an EO-encoded "char": value + 1 (the server reads
            // it via get_char() which decodes the number). Values 0..=252 encode
            // to a single byte.
            payload.put_u8((seq + 1) as u8);
        }
        payload.put_slice(data);

        if self.client_encryption_multiple != 0 {
            encrypt_packet(&mut payload, self.client_encryption_multiple);
        }

        let length_bytes = encode_number(payload.len() as i32)?;
        let mut out = BytesMut::with_capacity(2 + payload.len());
        out.put_slice(&length_bytes[0..2]);
        out.put(payload);
        self.stream.write_all(&out).await?;
        Ok(())
    }

    async fn send<T: EoSerialize>(
        &mut self,
        action: PacketAction,
        family: PacketFamily,
        packet: &T,
    ) -> Result<()> {
        let mut writer = EoWriter::new();
        packet.serialize(&mut writer)?;
        self.send_packet(action, family, &writer.to_byte_array())
            .await
    }

    async fn recv(&mut self) -> Result<(PacketAction, PacketFamily, Bytes)> {
        let mut len_buf = [0u8; 2];
        self.stream.read_exact(&mut len_buf).await?;
        let len = decode_number(&len_buf) as usize;
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).await?;
        if self.server_encryption_multiple != 0 {
            decrypt_packet(&mut buf, self.server_encryption_multiple);
        }
        let action = PacketAction::from(buf[0]);
        let family = PacketFamily::from(buf[1]);
        let data = Bytes::copy_from_slice(&buf[2..]);
        Ok((action, family, data))
    }

    async fn init_handshake(&mut self) -> Result<()> {
        let challenge = rand::rng().random_range(0..=1_000_000);
        let init = InitInitClientPacket {
            challenge,
            version: VERSION,
            hdid: String::new(),
        };
        self.send(PacketAction::Init, PacketFamily::Init, &init)
            .await?;

        let (_, _, data) = self.recv().await?;
        let reply = InitInitServerPacket::deserialize(&EoReader::new(data))?;
        if reply.reply_code != InitReply::OK {
            return Err(anyhow!("init failed: {:?}", reply.reply_code));
        }
        let Some(InitInitServerPacketReplyCodeData::OK(ok)) = reply.reply_code_data else {
            return Err(anyhow!("init reply missing OK data"));
        };
        if ok.challenge_response != server_verification_hash(challenge) {
            return Err(anyhow!("server verification failed"));
        }

        let start = get_init_sequence_start(ok.seq1 as i32, ok.seq2 as i32);
        tracing::debug!(
            seq1 = ok.seq1,
            seq2 = ok.seq2,
            start,
            "init handshake complete"
        );
        // The sequencer already advanced once for the Init send; only reset the
        // start value (not the counter), matching the reference client.
        self.sequencer.set_start(start);
        self.client_encryption_multiple = ok.client_encryption_multiple;
        self.server_encryption_multiple = ok.server_encryption_multiple;
        self.player_id = ok.player_id;
        Ok(())
    }

    async fn connection_accept(&mut self) -> Result<()> {
        let accept = ConnectionAcceptClientPacket {
            client_encryption_multiple: self.client_encryption_multiple as i32,
            server_encryption_multiple: self.server_encryption_multiple as i32,
            player_id: self.player_id,
        };
        self.send(PacketAction::Accept, PacketFamily::Connection, &accept)
            .await
    }

    async fn create_account(&mut self, username: &str, password: &str) -> Result<()> {
        self.send(
            PacketAction::Request,
            PacketFamily::Account,
            &AccountRequestClientPacket {
                username: username.to_string(),
            },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = AccountReplyServerPacket::deserialize(&EoReader::new(data))?;
        let session_id = match reply.reply_code {
            AccountReply::Unrecognized(id) => id,
            AccountReply::Exists => return Ok(()),
            other => return Err(anyhow!("account request failed: {other:?}")),
        };

        self.send(
            PacketAction::Create,
            PacketFamily::Account,
            &AccountCreateClientPacket {
                session_id,
                username: username.to_string(),
                password: password.to_string(),
                full_name: username.to_string(),
                location: "Botland".to_string(),
                email: format!("{username}@bots.test"),
                computer: "bot".to_string(),
                hdid: "bot".to_string(),
            },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = AccountReplyServerPacket::deserialize(&EoReader::new(data))?;
        match reply.reply_code {
            AccountReply::Created | AccountReply::Exists => Ok(()),
            other => Err(anyhow!("account create failed: {other:?}")),
        }
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<Vec<i32>> {
        self.send(
            PacketAction::Request,
            PacketFamily::Login,
            &LoginRequestClientPacket {
                username: username.to_string(),
                password: password.to_string(),
            },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = LoginReplyServerPacket::deserialize(&EoReader::new(data))?;
        match reply.reply_code {
            LoginReply::OK => {}
            other => return Err(anyhow!("login failed: {other:?}")),
        }
        let Some(LoginReplyServerPacketReplyCodeData::OK(ok)) = reply.reply_code_data else {
            return Err(anyhow!("login reply missing OK data"));
        };
        tracing::debug!(
            chars = ?ok.characters.iter().map(|c| (&c.name, c.id)).collect::<Vec<_>>(),
            "login ok"
        );
        Ok(ok.characters.iter().map(|c| c.id).collect())
    }

    async fn create_character(&mut self, name: &str) -> Result<i32> {
        self.send(
            PacketAction::Request,
            PacketFamily::Character,
            &CharacterRequestClientPacket {
                request_string: "NEW".to_string(),
            },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = CharacterReplyServerPacket::deserialize(&EoReader::new(data))?;
        let session_id = match reply.reply_code {
            CharacterReply::Unrecognized(id) => id,
            other => return Err(anyhow!("character request failed: {other:?}")),
        };

        self.send(
            PacketAction::Create,
            PacketFamily::Character,
            &CharacterCreateClientPacket {
                session_id,
                gender: Gender::Male,
                hair_style: 1,
                hair_color: 1,
                skin: 1,
                name: name.to_string(),
            },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = CharacterReplyServerPacket::deserialize(&EoReader::new(data))?;
        if reply.reply_code != CharacterReply::OK {
            return Err(anyhow!("character create failed: {:?}", reply.reply_code));
        }
        let Some(CharacterReplyServerPacketReplyCodeData::OK(ok)) = reply.reply_code_data else {
            return Err(anyhow!("character create reply missing OK data"));
        };
        ok.characters
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.id)
            .ok_or_else(|| anyhow!("created character {name} not in list"))
    }

    async fn enter_game(&mut self, character_id: i32) -> Result<()> {
        self.send(
            PacketAction::Request,
            PacketFamily::Welcome,
            &WelcomeRequestClientPacket { character_id },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = WelcomeReplyServerPacket::deserialize(&EoReader::new(data))?;
        let session_id = match reply.welcome_code_data {
            Some(WelcomeReplyServerPacketWelcomeCodeData::SelectCharacter(d)) => d.session_id,
            _ => {
                return Err(anyhow!(
                    "unexpected welcome reply: {:?}",
                    reply.welcome_code
                ));
            }
        };

        self.send(
            PacketAction::Msg,
            PacketFamily::Welcome,
            &WelcomeMsgClientPacket {
                session_id,
                character_id,
            },
        )
        .await?;

        let (_, _, data) = self.recv().await?;
        let reply = WelcomeReplyServerPacket::deserialize(&EoReader::new(data))?;
        if matches!(
            reply.welcome_code_data,
            Some(WelcomeReplyServerPacketWelcomeCodeData::EnterGame(_))
        ) {
            Ok(())
        } else {
            Err(anyhow!(
                "unexpected enter-game reply: {:?}",
                reply.welcome_code
            ))
        }
    }

    async fn idle_loop(&mut self, seconds: u64) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(seconds);
        while tokio::time::Instant::now() < deadline {
            let remaining = deadline - tokio::time::Instant::now();
            match tokio::time::timeout(remaining, self.recv()).await {
                Ok(Ok((action, family, _))) => {
                    tracing::info!(?action, ?family, "received");
                }
                Ok(Err(e)) => {
                    tracing::warn!("recv error: {e}");
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

async fn run_bot(addr: String, username: String, char_name: String) -> Result<()> {
    let password = format!("{username}pw");

    let mut bot = Bot::connect(&addr).await?;
    bot.init_handshake().await?;
    bot.connection_accept().await?;
    bot.create_account(&username, &password).await?;

    let characters = bot.login(&username, &password).await?;
    let character_id = if characters.is_empty() {
        bot.create_character(&char_name).await?
    } else {
        characters[0]
    };

    bot.enter_game(character_id).await?;
    tracing::info!("{username}: entered game as {char_name} (id {character_id})");

    bot.idle_loop(10).await;
    Ok(())
}

/// Maps a non-negative index to a lowercase-letter string (a, b, ..., z, aa, ab, ...).
fn index_to_letters(mut n: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
        if n == 0 {
            break;
        }
    }
    out.iter().rev().collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut bots = 1usize;
    let mut host = "127.0.0.1:8078".to_string();
    let mut account_prefix = "bot".to_string();

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bots" => {
                bots = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(1);
                i += 1;
            }
            "--host" => {
                host = args.get(i + 1).cloned().unwrap_or(host);
                i += 1;
            }
            "--account-prefix" => {
                account_prefix = args.get(i + 1).cloned().unwrap_or(account_prefix);
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("spawning {bots} bots against {host}");

    let mut handles = Vec::new();
    for n in 0..bots {
        let username = format!("{account_prefix}{n}");
        let char_name = format!("{account_prefix}{}", index_to_letters(n));
        let addr = host.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = run_bot(addr, username.clone(), char_name).await {
                tracing::error!("{username}: {e}");
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    tracing::info!("done");
    Ok(())
}
