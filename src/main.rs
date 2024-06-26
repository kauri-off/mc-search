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

#[tokio::main]
async fn main() {
    println!("MC Search: Starting");
    let threads = 100;

    let (tx, rx) = mpsc::channel(5);
    let tx = Arc::new(Mutex::new(tx));

    let servers = update().await;
    let _handler = task::spawn(port_handler(rx));

    if let Ok(servers) = servers {
        for server in servers {
            tx.lock().await.send(server).await.unwrap();
        }
    }

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
}

#[cfg(test)]
mod tests {
    use crate::scanner::server_data::ServerData;

    #[tokio::test]
    async fn test_server() {
        let server_data = ServerData::from(&"127.0.0.1:25565".parse().unwrap()).await;
        if let Some(server_data) = server_data {
            eprintln!("{}", server_data);
        } else {
            assert!(false, "Server not responsing");
        }
    }
}
