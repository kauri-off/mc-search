use std::{fs::remove_file, net::SocketAddr, path::Path, sync::{Arc, Mutex as SyncMutex}, time::Duration};

use mc_scanner::scanner::recieve_port_open;
use mc_search::{check_tcp_port_open, get_random_ip_address};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    task,
};
mod mc_scanner;

#[tokio::main]
async fn main() {
    println!("MC Search: Starting");
    let (tx, rx) = mpsc::channel(5);
    let tx = Arc::new(Mutex::new(tx));

    let reciever_task = task::spawn(recieve_port_open(rx));
    let migrate_tx = Arc::clone(&tx);

    migrate(migrate_tx).await;

    let mut task_list = vec![];
    let threads = 4;
    for _ in 0..threads {
        let mutex_tx = Arc::clone(&tx);
        let task = task::spawn(start_checking(mutex_tx));
        task_list.push(task);
    }

    for task in task_list {
        task.await.unwrap();
    }

    reciever_task.await.unwrap();
}

async fn start_checking(tx: Arc<Mutex<Sender<SocketAddr>>>) {
    loop {
        let addr = get_random_ip_address();
        if !check_tcp_port_open(&addr) {
            println!("[-] {}", addr);
            continue;
        }
        let tx = tx.lock().await;
        tx.send(addr).await.unwrap();
    }
}

async fn migrate(tx: Arc<Mutex<Sender<SocketAddr>>>) {
    if !Path::new("ports.db").exists() {
        return;
    }
    let conn = tokio_rusqlite::Connection::open("ports.db").await.unwrap();
    let tx_list = vec![];
    let tx_list = Arc::new(SyncMutex::new(tx_list));
    let tx_list_clone = Arc::clone(&tx_list);
    conn.call(move |conn| {
        let mut stmt = conn.prepare("SELECT id, ip, port FROM ports").unwrap();
        let server_iter = stmt.query_map([], |row| {
            Ok(Server {
                ip: row.get(1)?,
                port: row.get(2)?,
            })
        })?;
        for server in server_iter {
            if let Ok(server) = server {
                tx_list_clone
                    .lock()
                    .unwrap()
                    .push(format!("{}:{}", server.ip, server.port));
            }
        }
        Ok(())
    })
    .await
    .unwrap();
    let tx_lock = tx.lock().await;
    for server in tx_list.lock().unwrap().iter() {
        tx_lock.send(server.parse().unwrap()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    conn.close().await.unwrap();
    remove_file("ports.db").unwrap();
}

struct Server {
    ip: String,
    port: i32,
}
