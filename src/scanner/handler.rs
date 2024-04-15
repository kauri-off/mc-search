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
            "CREATE TABLE IF NOT EXISTS 'mc_server' (
	'id'	     INTEGER NOT NULL,
	'ip'	     TEXT NOT NULL,
	'port'	     INTEGER NOT NULL,
	'version'    TEXT NOT NULL,
    'online'     INTEGER NOT NULL,
    'max_online' INTEGER NOT NULL,
    'motd'       TEXT NOT NULL,
    'license'    INTEGER,
	PRIMARY KEY('id' AUTOINCREMENT)
);",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS 'open_port' (
'id'	     INTEGER NOT NULL,
'ip'	     TEXT NOT NULL,
'port'	     INTEGER NOT NULL,
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
