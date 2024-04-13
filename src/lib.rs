use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use rand::Rng;
use tokio::{net::TcpStream, time::timeout};

pub fn get_random_ip_address() -> SocketAddr {
    let mut rng = rand::thread_rng();
    let ip_address = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
        )),
        25565,
    );
    ip_address
}

pub async fn check_tcp_port_open(addr: &SocketAddr) -> bool {
    let open = timeout(Duration::from_millis(1500), async {
        match TcpStream::connect(addr).await {
            Ok(_) => true,
            Err(_) => false,
        }
    })
    .await;
    if let Ok(open) = open {
        return open;
    }
    false
}
