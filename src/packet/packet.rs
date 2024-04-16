use mc_varint::{VarInt, VarIntWrite};
use std::io::{Cursor, Result};
use tokio::{io::AsyncReadExt, net::TcpStream};
use uuid::Uuid;

use mc_varint::VarIntRead;
#[derive(Debug, Clone, Copy)]
pub struct MinecraftUUID(pub Uuid);

impl MinecraftUUID {
    // Метод для кодирования UUID в байты
    pub fn to_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

// Определение структуры для пакета протокола Minecraft
pub struct MinecraftPacket {
    packet_id: u8,
    data: Vec<u8>,
}

impl MinecraftPacket {
    // Метод для получения данных пакета
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Добавляем длину пакета как VarInt
        let packet_length = VarInt::from(self.data.len() as i32 + 1); // +1 для packet_id
        let mut cur = Cursor::new(Vec::with_capacity(5));
        let _ = cur.write_var_int(packet_length);
        bytes.extend(cur.into_inner());

        // Добавляем packet_id
        bytes.push(self.packet_id);

        // Добавляем данные пакета
        bytes.extend(&self.data);

        bytes
    }
}

// Builder для построения пакета протокола Minecraft
pub struct MinecraftPacketBuilder {
    packet_id: u8,
    data: Vec<u8>,
}

impl MinecraftPacketBuilder {
    // Создание нового экземпляра Builder с заданным ID пакета
    pub fn new(packet_id: u8) -> Self {
        MinecraftPacketBuilder {
            packet_id,
            data: Vec::new(),
        }
    }

    // Добавление данных типа VarInt в пакет
    pub fn add_varint(&mut self, value: VarInt) -> &mut Self {
        let mut cur = Cursor::new(Vec::with_capacity(5));
        let _ = cur.write_var_int(value);

        self.add_bytes(&cur.into_inner());
        self
    }

    // Добавление данных типа String в пакет
    pub fn add_string(&mut self, value: &str) -> &mut Self {
        self.add_varint(VarInt::from(value.len() as i32))
            .add_bytes(value.as_bytes());
        self
    }

    // Добавление обычных байт в пакет
    pub fn add_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.data.extend_from_slice(bytes);
        self
    }

    pub fn add_uuid(&mut self, uuid: MinecraftUUID) -> &mut Self {
        self.data.extend_from_slice(uuid.to_bytes());
        self
    }

    // Завершение построения пакета
    pub fn build(&self) -> MinecraftPacket {
        MinecraftPacket {
            packet_id: self.packet_id,
            data: self.data.clone(),
        }
    }
}

pub async fn read_var_int(sock: &mut TcpStream) -> Result<VarInt> {
    let mut buf = Vec::new();

    loop {
        let temp = sock.read_u8().await?;
        buf.push(temp);

        if temp & 0b1000_0000 == 0 {
            let mut cur = Cursor::new(buf);
            let var_int = cur.read_var_int().unwrap();
            return Ok(var_int);
        }
    }
}
