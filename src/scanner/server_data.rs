use core::fmt;
use std::{net::SocketAddr, time::Duration};

use mc_query::status;
use rusqlite::{params, Connection};
use tokio::time::timeout;

use crate::scanner::scanner::LICENSE;

use super::scanner::license;

#[derive(Clone)]
pub struct ServerData {
    addr: SocketAddr,
    version: String,
    online: u32,
    max_online: u32,
    motd: String,
    license: Option<u8>,
}

impl ServerData {
    pub async fn from(addr: &SocketAddr) -> Option<Self> {
        let data = timeout(
            Duration::from_secs(3),
            status(&addr.ip().to_string(), addr.port()),
        )
        .await;

        let mut server_data = Option::None;
        if let Ok(data) = data {
            if let Ok(data) = data {
                server_data = Some(ServerData {
                    addr: addr.clone(),
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
        server_data
    }
    pub fn into_table(server_data: Option<ServerData>, conn: &mut Connection, addr: &SocketAddr) {
        if let Some(server_data) = server_data {
            conn.execute(
                    "INSERT INTO 'mc_server'(ip, port, version, online, max_online, motd, license) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![addr.ip().to_string(), addr.port(), server_data.version, server_data.online, server_data.max_online, server_data.motd, server_data.license],
                )
                .unwrap();
        } else {
            conn.execute(
                "INSERT INTO 'open_port'(ip, port) VALUES (?1, ?2)",
                params![addr.ip().to_string(), addr.port()],
            )
            .unwrap();
        }
    }
}

impl fmt::Display for ServerData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[+] |{}|:|{}| [{}] {}/{} | {}",
            self.license.unwrap_or(LICENSE::DISCONNECT.to_u8()),
            self.addr.ip(),
            self.version,
            self.online,
            self.max_online,
            self.motd
        )
    }
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
