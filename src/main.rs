use std::sync::Arc;

use tokio::{
    sync::{mpsc, Mutex},
    task,
};

use crate::{
    checker::{checker::check_random_ip, updater::update},
    scanner::handler::port_handler,
};
mod checker;
mod packet;
mod scanner;

fn main() {
    println!("MC Search: Starting");

    let threads = 32;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads + 2)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let (tx, rx) = mpsc::channel(5);
        let tx = Arc::new(Mutex::new(tx));

        let update_list = update().await;
        let _handler = task::spawn(port_handler(rx));

        if let Ok(servers) = update_list {
            for server in servers {
                tx.lock().await.send(server).await.unwrap();
            }
        }
        // tx.lock()
        //     .await
        //     .send("45.93.200.95:25565".parse().unwrap())
        //     .await
        //     .unwrap();
        // _handler.await.unwrap();

        loop {
            let mut task_list = vec![];
            for _ in 0..threads {
                let mutex_tx = Arc::clone(&tx);
                let task = task::spawn(check_random_ip(mutex_tx));
                task_list.push(task);
            }

            for task in task_list {
                task.await.expect("Failed to await task");
            }
        }
    });
}
