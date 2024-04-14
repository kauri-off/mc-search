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
                    Ok(t) => t,
                    Err(_) => false,
                }
            });
        }
    }

    let addr_c = addr.clone();
    let conn = conn.lock().await;
    conn.call(move |conn| {
            if let Some(server_data) = server_data {
                conn.execute(
                    "INSERT INTO 'ip'(ip, port, version, online, max_online, motd, license) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![addr_c.ip().to_string(), addr_c.port(), server_data.version, server_data.online, server_data.max_online, server_data.motd, server_data.license],
                )
                .unwrap();
                println!(
                    "[+] |{}|:|{}| [{}] {}/{} | {}",
                    server_data.license, addr.ip(), server_data.version, server_data.online, server_data.max_online, server_data.motd
                );
            } else {
                conn.execute(
                    "INSERT INTO 'ip'(ip, port) VALUES (?1, ?2)",
                    params![addr_c.ip().to_string(), addr_c.port()],
                )
                .unwrap();
            println!("[/] {}", addr.ip());
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
    license: bool
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

async fn license(addr: &SocketAddr, protocol: u16) -> std::io::Result<bool> {
    let mut sock = TcpStream::connect(addr).await?;
    let handshake = MinecraftPacketBuilder::new(0)
        .add_varint(VarInt(protocol as i32))
        .add_string(&addr.ip().to_string())
        .add_bytes(&protocol.to_be_bytes())
        .add_varint(VarInt(2))
        .build();

    // dbg!(handshake.to_bytes());
    sock.write(&handshake.to_bytes()).await.unwrap();

    let login = MinecraftPacketBuilder::new(0)
        .add_string("mcscannerbot")
        .add_uuid(MinecraftUUID(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        ))
        .build();

    sock.write(&login.to_bytes()).await.unwrap();

    let mut buf = Vec::new();
    sock.read_buf(&mut buf).await.unwrap();

    Ok(buf[1] != 3)
}
