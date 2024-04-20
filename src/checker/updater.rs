use std::{fs, net::SocketAddr, path::Path};

pub async fn update() -> Result<Vec<SocketAddr>, tokio_rusqlite::Error> {
    if !Path::new("./db/database.db").exists() {
        return Ok(vec![]);
    }
    println!("Updating...");

    let conn = tokio_rusqlite::Connection::open("./db/database.db")
        .await
        .unwrap();

    let res = conn.call(|conn| {
        let mut result = Vec::new();
        let mut stmt = conn
            .prepare("SELECT ip, port FROM 'mc_server' UNION SELECT ip, port FROM 'open_port'")?;
        let server_iter = stmt.query_map([], |row| {
            let ip: String = row.get(0)?;
            Ok(SocketAddr::new(ip.parse().unwrap(), row.get(1)?))
        })?;
        for server in server_iter {
            if let Ok(server) = server {
                result.push(server);
            }
        }
        Ok(result)
    })
    .await;
    conn.close().await.unwrap();

    fs::remove_file("./db/database.db").unwrap();
    res
}
