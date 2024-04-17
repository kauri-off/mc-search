use std::{net::SocketAddr, path::Path};

pub async fn update() -> Result<Vec<SocketAddr>, tokio_rusqlite::Error> {
    if !Path::new("./db/database.db").exists() {
        return Ok(vec![]);
    }
    println!("Updating...");

    let conn = tokio_rusqlite::Connection::open("./db/database.db")
        .await
        .unwrap();

    conn
        .call(|conn| {
            let mut result = Vec::new();
            let mut stmt = conn.prepare("SELECT ip, port FROM 'mc_server'")?;
            let server_iter = stmt.query_map([], |row| {
                let ip: String = row.get(0)?;
                Ok(SocketAddr::new(ip.parse().unwrap(), row.get(1)?))
            })?;
            for server in server_iter {
                if let Ok(server) = server {
                    result.push(server);
                }
            }
            conn.execute("DELETE FROM 'mc_server';", []).unwrap();
            Ok(result)
        })
        .await
}
