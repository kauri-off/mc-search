use std::{net::SocketAddr, sync::Arc, time::Duration};

use mc_query::status;
use rusqlite::params;
use tokio::{sync::Mutex, time::timeout};
use tokio_rusqlite::Connection;

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
            });
        }
    }

    let addr_c = addr.clone();
    let conn = conn.lock().await;
    conn.call(move |conn| {
            if let Some(server_data) = server_data {
                conn.execute(
                    "INSERT INTO 'ip'(ip, port, version, online, max_online, motd) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![addr_c.ip().to_string(), addr_c.port(), server_data.version, server_data.online, server_data.max_online, server_data.motd],
                )
                .unwrap();
                println!(
                    "[+] [{}] {}/{} | {}",
                    server_data.version, server_data.online, server_data.max_online, server_data.motd
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
