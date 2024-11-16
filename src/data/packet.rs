// header data:
// byte 0: type of message
//  0xF0 -> name assignment
//  0xF1 -> message send
// bytes 1-7 (inclusive): reserved for future use
// everything else: message data
// PACKET MUST END WITH EOF (0x04)

#[repr(u8)]
pub enum PacketType {
  NameAssign = 0xF0,
}

#[repr(u8)]
pub enum PacketSymbols {
  Eof = 0x04,
}

pub fn create_id_packet(id: String) -> Vec<u8> {
  let mut packet: Vec<u8> = vec![0u8; 8]; // header
  packet[0] = PacketType::NameAssign as u8;
  
  for byte in id.as_bytes() {
    packet.push(*byte);
  }
  
  packet.push(PacketSymbols::Eof as u8);
  return packet;
}