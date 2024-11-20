use std::net::TcpStream;

pub trait ServerPackets {
  fn forward(stream: &mut TcpStream) -> std::io::Result<()>;
}