use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::data::generate_unique_id;
use crate::data::packet::create_id_packet;

pub fn thread_stream(mut stream: TcpStream, taken: Arc<Mutex<HashMap<String, SocketAddr>>>) -> JoinHandle<()> {
  thread::spawn(move || {
    let peer_addr = stream.peer_addr().unwrap();
    println!("Connection from {}", peer_addr);
    
    let id = match generate_unique_id(peer_addr, taken.clone()) {
      Ok(id) => id,
      Err(e) => {
        println!("Error generating ID: {}", e);
        return;
      }
    };
    
    let packet = create_id_packet(id);
    match stream.write_all(packet.as_slice()) {
      Ok(_) => {
        println!("Successfully sent packet to {}", peer_addr);
      }
      Err(e) => {
        println!("Error writing to stream: {}", e);
        return;
      }
    }
  })
}