use std::{net::SocketAddr, sync::Arc};

use mc_search::{check_tcp_port_open, get_random_ip_address};
use tokio::sync::{mpsc::Sender, Mutex};

pub async fn check_random_ip(tx: Arc<Mutex<Sender<SocketAddr>>>) {
    let addr = get_random_ip_address();
    check_ip(tx, addr).await;
}

pub async fn check_ip(tx: Arc<Mutex<Sender<SocketAddr>>>, addr: SocketAddr) {
    if check_tcp_port_open(&addr).await {
        let tx = tx.lock().await;
        if let Err(e) = tx.send(addr).await {
            eprintln!("Failed to send address over channel: {:?}", e);
        }
    } else {
        // println!("[-] {}", addr);
    }
}
