use std::{net::SocketAddr, sync::Arc};

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

fn main() {
    println!("MC Search: Starting");

    let threads = 14;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads + 2)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let (tx, rx) = mpsc::channel(5);
        let tx = Arc::new(Mutex::new(tx));

        let _ = task::spawn(recieve_port_open(rx));

        loop {
            let mut task_list = vec![];
            for _ in 0..threads {
                let mutex_tx = Arc::clone(&tx);
                let task = task::spawn(start_checking(mutex_tx));
                task_list.push(task);
            }

            for task in task_list {
                task.await.expect("Failed to await task");
            }
        }
    });
}

async fn start_checking(tx: Arc<Mutex<Sender<SocketAddr>>>) {
    let addr = get_random_ip_address();
    if check_tcp_port_open(&addr) {
        let tx = tx.lock().await;
        if let Err(e) = tx.send(addr).await {
            eprintln!("Failed to send address over channel: {:?}", e);
        }
    } else {
        // println!("[-] {}", addr);
    }
}
