use mc_search::{check_tcp_port_open, get_random_ip_address};
use std::sync::{Arc, Mutex};
use std::thread;
use rusqlite::{params, Connection};

fn main() -> Result<(), rusqlite::Error> {
    // Создаем соединение с SQLite базой данных
    let conn = Connection::open("ports.db")?;

    // Создаем таблицу для хранения результатов проверки портов, если она не существует
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ports (
                  id INTEGER PRIMARY KEY,
                  ip TEXT NOT NULL,
                  port INTEGER NOT NULL,
                  status TEXT NOT NULL
                  )",
        [],
    )?;

    let conn_mutex = Arc::new(Mutex::new(conn));
    let num_threads = 32; // Количество потоков, которые вы хотите использовать
    let mut handles = vec![];
    println!("Starting scanning network for open port: 25565");

    for _ in 0..num_threads {
        let conn_mutex = Arc::clone(&conn_mutex);
        let handle = thread::spawn(move || {
            loop {
                let ip = get_random_ip_address();
                if !check_tcp_port_open(&ip) {
                    // println!("[-] {}", ip);
                    continue;
                }
                println!("[+] {}", ip);
                let conn = conn_mutex.lock().unwrap();
                conn.execute(
                    "INSERT INTO ports (ip, port, status) VALUES (?1, ?2, ?3)",
                    params![format!("{}", ip.ip()), ip.port(), "open"],
                )
                .unwrap();
            }
        });
        handles.push(handle);
    }

    // Дожидаемся завершения всех потоков
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
