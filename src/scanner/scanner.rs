use mc_varint::VarInt;
use std::{net::SocketAddr, sync::Arc};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use tokio_rusqlite::Connection as TConnection;
use uuid::Uuid;

use crate::packet::packet::{MinecraftPacket, MinecraftPacketBuilder, MinecraftUUID};

use super::server_data::ServerData;

pub async fn scan_server(addr: SocketAddr, conn: Arc<Mutex<TConnection>>) {
    let server_data = ServerData::from(&addr).await;
    if let Some(server_data) = server_data.clone() {
        println!("{}", server_data);
    }

    let conn = conn.lock().await;
    conn.call(move |conn| {
        ServerData::into_table(server_data, conn, &addr);
        Ok(())
    })
    .await
    .unwrap();
}

pub async fn license(addr: &SocketAddr, protocol: u16) -> std::io::Result<Option<LICENSE>> {
    let mut sock = TcpStream::connect(addr).await?;
    let handshake = MinecraftPacketBuilder::new(0)
        .add_varint(VarInt::from(protocol as i32))
        .add_string(&addr.ip().to_string())
        .add_bytes(&protocol.to_be_bytes())
        .add_varint(VarInt::from(2))
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
    let packet = MinecraftPacket::from(&mut sock).await;

    match packet {
        Ok(packet) => Ok(LICENSE::from_u8(&packet.packet_id)),
        Err(t) => Err(t),
    }
}

#[derive(Clone, Copy)]
pub enum LICENSE {
    DISCONNECT = 0x00,
    LICENSE = 0x01,
    CRACKED = 0x03,
}

impl LICENSE {
    fn from_u8(value: &u8) -> Option<LICENSE> {
        match value {
            0x00 => Some(LICENSE::DISCONNECT),
            0x01 => Some(LICENSE::LICENSE),
            0x03 => Some(LICENSE::CRACKED),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
