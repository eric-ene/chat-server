use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use chat_shared::packet::assign::{AssignPacket, NameResponse, NameResponsePacket};
use chat_shared::packet::{self, Packet, PacketSymbols, ProcessedPacket};
use chat_shared::{packet::handshake::HandshakeStatus, stream::read::{ReadError, ReadUntil}};
use chat_shared::packet::handshake::HandshakePacket;
use crate::data::generate_unique_id;
use crate::packet::assign::NameError;
use crate::packet::handshake::{hs_packet_from_err, HandshakeError};
use crate::packet::Process;


pub type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
pub type SharedStream = Arc<Mutex<TcpStream>>;

pub fn thread_stream(
  mut stream: TcpStream,
  ids: SharedMap<String, SharedStream>,
  usernames: SharedMap<String, String>,
) -> JoinHandle<()> {
  return thread::spawn(move || {
    let stream_mutex = Arc::new(Mutex::new(stream));
    
    let mut stream = stream_mutex.lock().unwrap();
    
    stream.set_write_timeout(Some(Duration::from_secs_f32(10.0))).unwrap();
    let peer_addr = stream.peer_addr().unwrap();
    println!("Connection from {}", peer_addr);
    
    let id = match generate_unique_id(stream_mutex.clone(), ids.clone()) {
      Ok(id) => id,
      Err(e) => {
        println!("Error generating ID: {}", e);
        return;
      }
    };
    
    let assign = AssignPacket {
      content: id,
    };
    
    loop {
      let mut buf = Vec::new();

      let n_read = stream.read_until_symbol(&mut buf, PacketSymbols::Eom).unwrap_or_else(|e| {
        println!("couldn't read from buffer! {}", e);
        0
      });

      if n_read == 0 {
        println!("connection closed.");
        return;
      }

      if n_read < 8 {
        println!("mangled packet! size {} is too small!", n_read);
        continue;
      }

      let packet = Packet::from_bytes(&mut buf);
      let processed_packet = match packet.process() {
        Ok(pack) => pack,
        Err(e) => {
          println!("couldn't process packet! {}", e);
          continue;
        }
      };

      match processed_packet {
        ProcessedPacket::AssignRequest(_) => break,
        _ => {
          println!("received a non-assn request packet too early! waiting for assn request packet...");
          continue;
        }
      }
    }

    let packet = ProcessedPacket::new_raw(ProcessedPacket::Assign(assign));
    match stream.write_all(packet.as_slice()) {
      Ok(_) => {
        println!("Successfully sent packet to {}", peer_addr);
      }
      Err(e) => {
        println!("Error writing to stream (connection likely closed): {}", e);
        return;
      }
    }
    
    stream.set_nonblocking(true).unwrap();
    let mut buf = Vec::new();
    let timeout = Duration::from_secs_f32(0.5);

    // main event loop
    drop(stream);
    
    loop {
      let mut stream = match stream_mutex.lock() {
        Ok(stream) => stream,
        Err(e) => {
          println!("Error locking stream! Exiting thread and closing connection...");
          return;
        }
      };
      
      let n_read = match stream.read_until_timeout(&mut buf, PacketSymbols::Eom, timeout) {
        Ok(n) => n,
        Err(e) => match e {
          ReadError::Timeout => continue,
          ReadError::Other(e) => {
            println!("couldn't read from stream! {}", e);
            buf = Vec::new();
            continue;
          }
        }
      };
      
      drop(stream);

      if n_read == 0 {
        println!("connection closed.");
        return;
      }

      if n_read < 8 {
        println!("mangled packet! size {} is too small!", n_read);
        continue;
      }

      let packet = Packet::from_bytes(&mut buf);

      // deserialize packet
      let processed = match packet.process() {
        Ok(processed) => processed,
        Err(e) => {
          println!("couldn't process packet: {}", e);
          continue;
        }
      };

      // do stuff with the processed packet
      // lock stream
      match processed {
        ProcessedPacket::NameRequest(packet) => {
          let response = match packet.process((ids.clone(), usernames.clone())) {
            Ok(_) => NameResponse::Success,
            Err(e) => NameResponse::Failure(match e {
              NameError::NameTaken(err) => err,
              NameError::LockError(err) => format!("Server error: {}", err),
            })
          };

          let packet = NameResponsePacket {
            status: response
          };

          let packet_raw = ProcessedPacket::new_raw(ProcessedPacket::NameResponse(packet));

          let mut stream = match stream_mutex.lock() {
            Ok(stream) => stream,
            Err(e) => {
              println!("Error locking stream! Exiting thread and closing connection...");
              return;
            }
          };
          
          match stream.write_all(&packet_raw) {
            Ok(_) => println!("successfully sent name response to {}.", peer_addr),
            Err(e) => println!("error sending name response: {}", e)
          }
        }
        ProcessedPacket::Message(packet) => {

        }
        ProcessedPacket::Handshake(packet) => {
          let dst = packet.dst.clone();
          
          let (forward, dst_stream) = match packet.process((dst, ids.clone(), usernames.clone())) {
            Ok((ret_pack, ret_sream)) => (ret_pack, Some(ret_sream)),
            Err(e) => (hs_packet_from_err(e), None)
          };
          
          let status = forward.status.clone();
          let packet_raw = ProcessedPacket::new_raw(ProcessedPacket::Handshake(forward));
          
          match dst_stream {
            Some(other) => { // forward to recipient
              println!("locking stream...");
              
              let mut stream = match other.lock() {
                Ok(stream) => stream,
                Err(e) => {
                  println!("error locking returned tcp stream! continuing...");
                  continue;
                }
              };
              
              match stream.write_all(&packet_raw) {
                Ok(_) => println!("successfully forwarded handshake ({:?}) to {}.", status, peer_addr),
                Err(e) => println!("error sending handshake response: {}", e)
              }
            }
            None => { // error happened, return to sender
              let mut stream = match stream_mutex.lock() {
                Ok(stream) => stream,
                Err(e) => {
                  println!("Error locking stream! Exiting thread and closing connection...");
                  return;
                }
              };
              
              match stream.write_all(&packet_raw) {
                Ok(_) => println!("successfully sent handshake response to {}.", peer_addr),
                Err(e) => println!("error sending handshake response: {}", e)
              }
            }
          }
        }
        packet => {
          println!("ignoring packet: {:?}", packet);
        }
      }
    }
  });
}