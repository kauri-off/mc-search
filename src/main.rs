use std::{
    net::SocketAddr,
    sync::Arc
};

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

    let mut task_list = vec![];
    let threads = 8;
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
        if check_tcp_port_open(&addr) {
            let tx = tx.lock().await;
            tx.send(addr).await.unwrap();
        } else {
            println!("[-] {}", addr);
        }
    }
}