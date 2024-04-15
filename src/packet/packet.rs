use tokio::{io::AsyncReadExt, net::TcpStream};
use uuid::Uuid;
use std::io::Result;

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
        let packet_length = VarInt(self.data.len() as i32 + 1); // +1 для packet_id
        bytes.extend(packet_length.to_bytes());

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
        self.data.extend(value.to_bytes());
        self
    }

    // Добавление данных типа String в пакет
    pub fn add_string(&mut self, value: &str) -> &mut Self {
        self.add_varint(VarInt(value.len() as i32))
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

// Константы для VarInt
const CONTINUE_BIT: u8 = 0b1000_0000;
const SEGMENT_BITS: u32 = 0b0111_1111;

// Тип VarInt
#[derive(Debug, Clone, Copy)]
pub struct VarInt(pub i32);

impl VarInt {
    pub async fn from_socket(stream: &mut TcpStream) -> Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut current_byte: u8;

        loop {
            current_byte = stream.read_u8().await?;
            value |= ((current_byte & 0x7F) as i32) << position;

            if (current_byte & 0x80) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                return Err(std::io::Error::new(
                     std::io::ErrorKind::InvalidData,
                                   "VarInt is too large",
                     ));
            }
        }

        Ok(VarInt(value))
    }
    // Метод для кодирования VarInt в байты
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut value = self.0 as u32;
        let mut bytes = Vec::new();

        loop {
            let mut byte = (value & SEGMENT_BITS) as u8;
            value >>= 7;

            if value != 0 {
                byte |= CONTINUE_BIT;
            }

            bytes.push(byte);

            if value == 0 {
                break;
            }
        }

        bytes
    }
}
