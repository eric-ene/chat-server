use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use std::thread::JoinHandle;
use chat_shared::packet::assign::{AssignPacket};
use chat_shared::packet::{Packet, ProcessedPacket};
use crate::data::generate_unique_id;

pub fn thread_stream(mut stream: TcpStream, taken: Arc<Mutex<HashMap<String, SocketAddr>>>) -> JoinHandle<()> {
  return thread::spawn(move || {
    let peer_addr = stream.peer_addr().unwrap();
    println!("Connection from {}", peer_addr);
    
    let id = match generate_unique_id(peer_addr, taken.clone()) {
      Ok(id) => id,
      Err(e) => {
        println!("Error generating ID: {}", e);
        return;
      }
    };
    
    let assign = AssignPacket {
      content: id,
    };
    
    let packet = ProcessedPacket::new_raw(ProcessedPacket::Assign(assign));
    match stream.write_all(packet.as_slice()) {
      Ok(_) => {
        println!("Successfully sent packet to {}", peer_addr);
      }
      Err(e) => {
        println!("Error writing to stream (connection closed): {}", e);
        return;
      }
    }
    
    // main event loop
    // loop {
    //   let mut buf = Vec::new();
    //   let n_read = match stream.read_to_end(&mut buf) {
    //     Ok(n) => n,
    //     Err(e) => {
    //       println!("error reading from stream: {}", e);
    //       continue;
    //     }
    //   };
    //   
    //   if n_read == 0 {
    //     println!("connection closed.");
    //     return;
    //   }
    // 
    //   if n_read < 8 {
    //     println!("mangled packet! size {} is too small!", n_read);
    //     continue;
    //   }
    //   
    //   let packet = Packet::from_bytes(buf);
    //   
    //   // deserialize packet
    //   let processed = match packet.process() {
    //     Ok(processed) => processed,
    //     Err(e) => {
    //       println!("couldn't process packet: {}", e);
    //       continue;
    //     }
    //   };
    //   
    //   // do stuff with the processed packet
    //   match processed {
    //     ProcessedPacket::Message(packet) => {
    //       
    //     }
    //     ProcessedPacket::Handshake(packet) => {
    //       
    //     }
    //     packet => {
    //       println!("ignoring packet: {:?}", packet);
    //     }
    //   }
    // }
  });
}