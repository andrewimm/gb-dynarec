//! Packet protocol for communication between the debugger and the emulator

pub struct Packet {
  packet_type: PacketType,
  body: Box<[u8]>,
}

pub enum PacketType {

}
