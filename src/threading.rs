use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use chat_shared::packet::assign::{AssignPacket, NameResponse, NameResponsePacket};
use chat_shared::packet::{self, Packet, PacketSymbols, ProcessedPacket};
use chat_shared::{packet::handshake::HandshakeStatus, stream::read::{ReadError, ReadUntil}};
use chat_shared::packet::encrypt::EncryptedPacket;
use chat_shared::packet::handshake::HandshakePacket;
use chat_shared::stream::read::ReadExact;
use chat_shared::stream::write::SharedWrite;
use rsa::RsaPrivateKey;
use rsa::traits::PublicKeyParts;
use eric_aes::{generate_key, rsatools};
use eric_aes::aestools::CryptError;
use crate::data::generate_unique_id;
use crate::packet::assign::NameError;
use crate::packet::handshake::{hs_packet_from_err, HandshakeError};
use crate::packet::Process;


pub type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
pub type SharedStream = chat_shared::stream::read::SharedStream;
pub type SharedKey = Arc<Vec<u8>>;

#[derive(Clone)]
pub struct Endpoint {
  pub stream: SharedStream,
  pub key: SharedKey,
}

pub fn thread_stream(
  mut stream: TcpStream,
  ids: SharedMap<String, Endpoint>,
  usernames: SharedMap<String, String>,
) -> JoinHandle<()> {
  return thread::spawn(move || {
    stream.set_write_timeout(Some(Duration::from_secs_f32(10.0))).unwrap();
    stream.set_nonblocking(true).unwrap();
    
    let peer_addr = stream.peer_addr().unwrap();
    println!("Connection from {}", peer_addr);
    
    let stream_mutex = Arc::new(Mutex::new(stream));
    
    let id = match generate_unique_id(stream_mutex.clone(), ids.clone()) {
      Ok(id) => id,
      Err(e) => {
        println!("Error generating ID: {}", e);
        return;
      }
    };
    
    let request_packet = loop {
      let mut buf = match stream_mutex.read_packet() {
        Ok(vec) => vec,
        Err(e) => {
          println!("Couldn't read packet: {}", e);
          return;
        }
      };

      let packet = Packet::from_bytes(&mut buf);
      let processed_packet = match packet.process() {
        Ok(pack) => pack,
        Err(e) => {
          println!("couldn't process packet! {}", e);
          continue;
        }
      };

      match processed_packet {
        ProcessedPacket::AssignRequest(pack) => break pack,
        _ => {
          println!("received a non-assn request packet too early! waiting for assn request packet...");
          continue;
        }
      }
    };
    
    let aes_key = generate_key();
    let aes_shared = Arc::new(aes_key.clone());
    
    let endp = Endpoint {
      stream: stream_mutex.clone(),
      key: aes_shared.clone(),
    };
    
    let mut ids_guard = match ids.lock() {
      Ok(guard) => guard,
      Err(e) => { println!("{e}"); return; }
    };
    
    ids_guard.insert(id.clone(), endp);
    drop(ids_guard);
    
    let assign = AssignPacket {
      content: id,
      aes_key: aes_key.clone()
    };

    let client_e = request_packet.e;
    let client_n = request_packet.n;
    
    let packet = ProcessedPacket::Assign(assign).new_raw_rsa(&client_e, &client_n);

    match stream_mutex.write_all_shared(&packet) {
      Ok(_) => {
        println!("Successfully sent packet to {}", peer_addr);
      }
      Err(e) => {
        println!("Error writing to stream (connection likely closed): {}", e);
        return;
      }
    }
    
    // main event loop
    loop {
      let mut raw = match stream_mutex.read_packet() {
        Ok(packet) => packet,
        Err(e) => {
          println!("Couldn't read from stream! {}", e);
          continue;
        }
      };     
      
      let packet = match Packet::from_aes_bytes(&mut raw, &aes_shared) {
        Ok(pack) => pack,
        Err(e) => {
          println!("Couldn't decrypt packet! {:?}", e);
          continue;
        }
      };

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

          let packet_raw = match ProcessedPacket::new_raw_aes(ProcessedPacket::NameResponse(packet), &aes_shared) {
            Ok(pack) => pack,
            Err(e) => {
              println!("Couldn't encrypt packet! {:?}", e);
              continue;
            }
          };

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
          let (forward, endpoint) = match packet.process((ids.clone(), usernames.clone())) {
            Ok((ret_pack, ret_sream)) => (ret_pack, ret_sream),
            Err(e) => { 
              println!("{:?}", e);
              continue;
            }
          };

          println!("locking stream...");

          // TODO: This blocks forever when other isn't the sender. Fix it.
          let mut stream = match endpoint.stream.lock() {
            Ok(stream) => stream,
            Err(e) => {
              println!("error locking returned tcp stream! continuing...");
              continue;
            }
          };

          let packet_raw = match ProcessedPacket::new_raw_aes(ProcessedPacket::Message(forward), &endpoint.key) {
            Ok(pack) => pack,
            Err(e) => {
              println!("Couldn't encrypt packet! {:?}", e);
              continue;
            }
          };

          println!("stream locked. sending...");

          match stream.write_all(&packet_raw) {
            Ok(_) => println!("successfully forwarded message."),
            Err(e) => println!("error sending handshake response: {}", e)
          }
        }
        ProcessedPacket::Handshake(packet) => {
          let dst = packet.dst.clone();
          
          let (forward, dst_stream) = match packet.process((dst, ids.clone(), usernames.clone())) {
            Ok((ret_pack, ret_sream)) => (ret_pack, Some(ret_sream)),
            Err(e) => (hs_packet_from_err(e), None)
          };
          
          
          let status = forward.status.clone();
          
          match dst_stream {
            Some(other) => { // forward to recipient
              println!("locking stream...");
              
              // TODO: This blocks forever when other isn't the sender. Fix it.
              let mut stream = match other.stream.lock() {
                Ok(stream) => stream,
                Err(e) => {
                  println!("error locking returned tcp stream! continuing...");
                  continue;
                }
              };

              let packet_raw = match ProcessedPacket::new_raw_aes(ProcessedPacket::Handshake(forward), &other.key) {
                Ok(pack) => pack,
                Err(e) => {
                  println!("Couldn't encrypt packet! {:?}", e);
                  continue;
                }
              };
              
              println!("stream locked. sending...");
              
              match stream.write_all(&packet_raw) {
                Ok(_) => println!("successfully forwarded handshake ({:?}).", status),
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

              let packet_raw = match ProcessedPacket::new_raw_aes(ProcessedPacket::Handshake(forward), &aes_shared) {
                Ok(pack) => pack,
                Err(e) => {
                  println!("Couldn't encrypt packet! {:?}", e);
                  continue;
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