use std::net::SocketAddr;
use tokio_rusqlite::Error;

pub async fn update() -> Result<Vec<SocketAddr>, Error> {
    println!("Updating...");

    let conn = tokio_rusqlite::Connection::open("./db/database.db")
        .await
        .unwrap();

    let servers = conn
        .call(|conn| {
            let mut result = Vec::new();
            let mut stmt = conn.prepare("SELECT * FROM 'mc_server'")?;
            let server_iter = stmt.query_map([], |row| {
                let ip: String = row.get(1)?;
                Ok(SocketAddr::new(ip.parse().unwrap(), row.get(2)?))
            })?;
            for server in server_iter {
                if let Ok(server) = server {
                    result.push(server);
                }
            }
            conn.execute("DELETE FROM 'mc_server';", []).unwrap();
            Ok(result)
        })
        .await;
    servers
}
