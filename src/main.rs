mod threading;
mod data;
mod packet;
mod networking;

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

const PORT: u16 = 8081;

fn main() {
    let mut handles = Vec::new();
    
    // mappings:
    //  key -> id (three-word-sequence)
    //  val -> socket address
    let mappings = Arc::new(Mutex::new(HashMap::new()));
    
    // usernames:
    //  key -> username (arbitrary_string)
    //  val -> id (three-word-sequence)
    let usernames = Arc::new(Mutex::new(HashMap::new()));
    
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    println!("listening on port {}...", PORT);
    for stream in listener.incoming() {
        println!("new connection");
        match stream {
            Ok(stream) => {
                let handle = threading::thread_stream(stream, mappings.clone(), usernames.clone());
                handles.push(handle);
            },
            Err(e) => println!("Error: {}", e),
        }
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
