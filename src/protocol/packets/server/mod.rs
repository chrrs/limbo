use self::status::ServerStatusPacket;

pub mod status;

#[derive(Debug)]
pub enum ServerPacket {
    Status(ServerStatusPacket),
}
