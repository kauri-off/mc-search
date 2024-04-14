use std::{net::SocketAddr, sync::Arc};

use tokio::{
    sync::{mpsc::Receiver, Mutex},
    task,
};

use super::scanner::scan_server;

pub async fn port_handler(mut rx: Receiver<SocketAddr>) {
    let conn = tokio_rusqlite::Connection::open("./db/database.db")
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
    'license'    BOOLEAN,
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
        task::spawn(scan_server(addr, conn_clone));
    }
}
