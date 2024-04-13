pub mod scanner {
    use mc_query::status;
    use rusqlite::params;
    use std::{net::SocketAddr, sync::Arc, time::Duration};
    use tokio::{sync::{mpsc::Receiver, Mutex}, task, time::timeout};
    use tokio_rusqlite::Connection;

    pub async fn recieve_port_open(mut rx: Receiver<SocketAddr>) {
        let conn = tokio_rusqlite::Connection::open("database.db")
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
        loop {
            let addr = rx.recv().await;
            if let Some(addr) = addr {
                let conn_clone = Arc::clone(&conn);
                task::spawn(do_staff(addr, conn_clone));
            }
        }
    }

    async fn do_staff(addr: SocketAddr, conn: Arc<Mutex<Connection>>) {
        let data = timeout(
            Duration::from_secs(3),
            status(&addr.ip().to_string(), addr.port()),
        ).await;
        let mut server_data = Option::None;
        if let Ok(data) = data {
            if let Ok(data) = data {
                println!(
                    "[+] [{}] {}/{} | {:?}",
                    data.version.name, data.players.online, data.players.max, data.motd
                );
                server_data = Some(ServerData {
                    version: data.version.name,
                    online: data.players.online,
                    max_online: data.players.max,
                    motd: format!("{:?}", data.motd)
                })
        }} else {
            println!("[/] {}", addr);
            server_data = None;
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
            } else {
                conn.execute(
                    "INSERT INTO 'ip'(ip, port) VALUES (?1, ?2)",
                    params![addr1.ip().to_string(), addr1.port()],
                )
                .unwrap();
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
        motd: String
    }
}
