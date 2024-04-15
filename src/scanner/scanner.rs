use mc_query::status;
use rusqlite::params;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
    time::timeout,
};
use tokio_rusqlite::Connection;
use uuid::Uuid;

use crate::packet::packet::{MinecraftPacketBuilder, MinecraftUUID, VarInt};

pub async fn scan_server(addr: SocketAddr, conn: Arc<Mutex<Connection>>) {
    let data = timeout(
        Duration::from_secs(3),
        status(&addr.ip().to_string(), addr.port()),
    )
    .await;

    let mut server_data = Option::None;
    if let Ok(data) = data {
        if let Ok(data) = data {
            server_data = Some(ServerData {
                version: data.version.name,
                online: data.players.online,
                max_online: data.players.max,
                motd: motd_fmt(data.motd),
                license: match license(&addr, data.version.protocol).await {
                    Ok(t) => match t {
                        Some(t) => Some(t.to_u8()),
                        None => None,
                    },
                    Err(_) => None,
                },
            });
        }
    }

    let addr_c = addr.clone();
    let conn = conn.lock().await;
    conn.call(move |conn| {
            if let Some(server_data) = server_data {
                conn.execute(
                    "INSERT INTO 'mc_server'(ip, port, version, online, max_online, motd, license) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![addr_c.ip().to_string(), addr_c.port(), server_data.version, server_data.online, server_data.max_online, server_data.motd, server_data.license],
                )
                .unwrap();
                println!(
                    "[+] |{}|:|{}| [{}] {}/{} | {}",
                    server_data.license.unwrap_or(LICENSE::DISCONNECT.to_u8()), addr.ip(), server_data.version, server_data.online, server_data.max_online, server_data.motd
                );
            } else {
                conn.execute(
                    "INSERT INTO 'open_port'(ip, port) VALUES (?1, ?2)",
                    params![addr_c.ip().to_string(), addr_c.port()],
                )
                .unwrap();
            // println!("[/] {}", addr.ip());
            }
            Ok(())
        })
        .await
        .unwrap();
}

struct ServerData {
    version: String,
    online: u32,
    max_online: u32,
    motd: String,
    license: Option<u8>,
}

pub fn motd_fmt(motd: mc_query::status::ChatObject) -> String {
    let motd = match motd {
        status::ChatObject::Object(t) => t.text,
        status::ChatObject::Array(_) => None,
        status::ChatObject::JsonPrimitive(t) => match t.as_str() {
            Some(t) => Some(String::from(t)),
            None => todo!(),
        },
    };
    match motd {
        Some(t) => t,
        None => String::from("error"),
    }
}

async fn license(addr: &SocketAddr, protocol: u16) -> std::io::Result<Option<LICENSE>> {
    let mut sock = TcpStream::connect(addr).await?;
    let handshake = MinecraftPacketBuilder::new(0)
        .add_varint(VarInt(protocol as i32))
        .add_string(&addr.ip().to_string())
        .add_bytes(&protocol.to_be_bytes())
        .add_varint(VarInt(2))
        .build();

    // dbg!(handshake.to_bytes());
    sock.write(&handshake.to_bytes()).await?;

    let login = MinecraftPacketBuilder::new(0)
        .add_string("mcscannerbot")
        .add_uuid(MinecraftUUID(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        ))
        .build();

    sock.write(&login.to_bytes()).await?;

    let len = VarInt::from_socket(&mut sock).await;
    if let Err(_) = len {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "connect error"));
    }
    let packet_id = sock.read_u8().await?;
    // dbg!(&packet_id);

    Ok(LICENSE::from_u8(&packet_id))
}

#[derive(Clone, Copy)]
enum LICENSE {
    DISCONNECT = 0x00,
    LICENSE = 0x01,
    CRACKED = 0x02
}


impl LICENSE {
    fn from_u8(value: &u8) -> Option<LICENSE> {
        match value {
            0x00 => Some(LICENSE::DISCONNECT),
            0x01 => Some(LICENSE::LICENSE),
            0x02 => Some(LICENSE::CRACKED),
            _ => None,
        }
    }

    fn to_u8(&self) -> u8 {
        *self as u8
    }
}