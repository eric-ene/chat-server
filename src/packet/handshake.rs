use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use chat_shared::packet::handshake::{HandshakePacket, HandshakeStatus};
use chat_shared::user::User;
use crate::packet::{validate_username, Process};
use crate::threading::{SharedMap, SharedStream};

pub enum HandshakeError {
  NotFound,
  ServerError,
}

impl Process for HandshakePacket {
  type OkType = (HandshakePacket, SharedStream);
  type ErrType = HandshakeError;
  type ArgsType = (
    String,
    SharedMap<String, SharedStream>,
    SharedMap<String, String>
  );

  /// - args.0: dst
  /// - args.1: ids
  /// - args.2: names
  fn process(self, args: Self::ArgsType) -> Result<Self::OkType, Self::ErrType> {
    let (dst, ids, names) = args;
    
    println!("validating username...");
    
    let stream = match validate_username(dst, ids, names) {
      Ok(shared_stream) => shared_stream,
      Err(e) => return Err(e)
    };
    
    println!("username validated.");
    
    let forwarded_packet = HandshakePacket {
      status: self.status,
      src: self.src,
      dst: String::new()
    };
    
    return Ok((forwarded_packet, stream))
  }
}

pub fn hs_packet_from_err(err: HandshakeError) -> HandshakePacket {
  let status = match err {
    HandshakeError::NotFound => HandshakeStatus::NotFound,
    HandshakeError::ServerError => HandshakeStatus::ServerError
  };
  
  let src = User {
    id: None,
    name: None,
  };
  
  return HandshakePacket {
    status,
    src,
    dst: String::new()
  }
}