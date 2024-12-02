pub mod assign;
pub mod handshake;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::packet::handshake::HandshakeError;
use crate::packet::handshake::HandshakeError::ServerError;
use crate::threading::{Endpoint, SharedMap, SharedStream};

pub trait Process {
  type OkType;
  type ErrType;
  type ArgsType;
  // Consume and process the processed packet.
  fn process(self, args: Self::ArgsType) -> Result<Self::OkType, Self::ErrType>;
}

pub fn validate_username(
  dst: String, 
  ids: SharedMap<String, Endpoint>, 
  users: SharedMap<String, String>
) -> Result<Endpoint, HandshakeError> {
  let mut users = match users.lock() {
    Ok(guard) => guard,
    Err(e) => {
      println!("{}", e);
      return Err(HandshakeError::ServerError)
    }
  };
  
  let true_dst = match users.get(&dst) {
    None => dst,
    Some(dst) => dst.clone()
  };
  
  drop(users);

  let ids = match ids.lock() {
    Ok(guard) => guard,
    Err(e) => {
      println!("{}", e);
      return Err(HandshakeError::ServerError)
    }
  };
  
  return match ids.get(&true_dst) {
    None => return Err(HandshakeError::NotFound),
    Some(shared) => Ok(shared.clone())
  }
}