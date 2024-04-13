pub mod scanner {
    use mc_query::status;
    use rusqlite::params;
    use std::{net::SocketAddr, sync::Arc, time::Duration};
    use tokio::{
        sync::{mpsc::Receiver, Mutex},
        task,
        time::timeout,
    };
    use tokio_rusqlite::Connection;

    pub async fn recieve_port_open(mut rx: Receiver<SocketAddr>) {
        let conn = tokio_rusqlite::Connection::open("/app/db/database.db")
            .await
            .unwrap();
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS 'ip' (
	'id'	     INTEGER NOT NULL,
	'ip'	     TEXT NOT NULL,
	'port'	     INTEGER NOT NULL,
	'version'    TEXT,
    'online'     INTEGER,
    'max_online' INTEGER,
    'motd'       TEXT,
	PRIMARY KEY('id' AUTOINCREMENT)
);",
                [],
            )
            .unwrap();
            Ok(())
        })
        .await
        .unwrap();
        let conn = Arc::new(Mutex::new(conn));
        while let Some(addr) = rx.recv().await {
            let conn_clone = Arc::clone(&conn);
            task::spawn(do_staff(addr, conn_clone));
        }
    }

    async fn do_staff(addr: SocketAddr, conn: Arc<Mutex<Connection>>) {
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
                    motd: formatter(data.motd),
                })
            }
        }
        let addr1 = addr.clone();
        let conn = conn.lock().await;
        conn.call(move |conn| {
            if let Some(server_data) = server_data {
                conn.execute(
                    "INSERT INTO 'ip'(ip, port, version, online, max_online, motd) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![addr1.ip().to_string(), addr1.port(), server_data.version, server_data.online, server_data.max_online, server_data.motd],
                )
                .unwrap();
                println!(
                    "[+] [{}] {}/{} | {}",
                    server_data.version, server_data.online, server_data.max_online, server_data.motd
                );
            } else {
                conn.execute(
                    "INSERT INTO 'ip'(ip, port) VALUES (?1, ?2)",
                    params![addr1.ip().to_string(), addr1.port()],
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

    pub fn formatter(motd: mc_query::status::ChatObject) -> String {
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
}
