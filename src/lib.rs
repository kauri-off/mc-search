use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

use rand::Rng;

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

pub fn check_tcp_port_open(addr: &SocketAddr) -> bool {
    match TcpStream::connect_timeout(addr, Duration::from_millis(1500)) {
        Ok(_) => true, // Соединение успешно установлено, порт открыт
        Err(_) => false, // Соединение не удалось, порт закрыт или недоступен
    }
}
