use std::collections::HashMap;
use std::fmt::format;
use std::net::SocketAddr;
use std::sync::{Arc, LockResult, Mutex};
use crate::packet::Process;
use chat_shared::packet::assign::NameRequestPacket;
use crate::threading::{Endpoint, SharedMap, SharedStream};

pub enum NameError {
  NameTaken(String),
  LockError(String),
}

impl Process for NameRequestPacket {

  type OkType = ();
  
  type ErrType = NameError;
  type ArgsType = (
    SharedMap<String, Endpoint>,
    SharedMap<String, String>,
  );

  fn process(self, args: Self::ArgsType) -> Result<Self::OkType, Self::ErrType> {
    let (ids, users) = args;
    
    let mut users = match users.lock() {
      Ok(guard) => guard,
      Err(e) => {
        println!("{}", e);
        return Err(NameError::LockError(format!("error locking users: {}", e)))
      }
    };

    let ids = match ids.lock() {
      Ok(guard) => guard,
      Err(e) => {
        println!("{}", e);
        return Err(NameError::LockError(format!("error locking ids: {}", e)))
      }
    };
    
    if users.contains_key(&self.content) {
      return Err(NameError::NameTaken(String::from("Name taken.")));
    }
    
    if !ids.contains_key(&self.sender) {
      return Err(NameError::NameTaken(String::from("ID not in database.")));
    }
    
    users.insert(self.content, self.sender);
    
    return Ok(());
  }
}